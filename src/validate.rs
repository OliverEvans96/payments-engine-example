use crate::account::AccountAccess;
use crate::account::{BaseAccountFeatures, UnlockedAccountFeatures};
use crate::currency::CurrencyFloat;
use crate::state::{AccountsState, DisputesState, TransactionsState};
use crate::types::TransactionError;
use crate::types::{Deposit, Dispute, PostDispute, Withdrawal};
use crate::types::{Disputable, Transaction, TransactionId};

fn check_for_duplicate_tx_id(
    tx_id: TransactionId,
    transactions: &TransactionsState,
) -> Result<(), TransactionError> {
    // TODO: Efficiently record duplicate transactions?
    if transactions.tx_exists(tx_id) {
        // Duplicate transactions are a bad sign
        Err(TransactionError::DuplicateTxId { tx: tx_id })
    } else {
        Ok(())
    }
}

fn check_for_positive_amount(
    tx: TransactionId,
    amount: CurrencyFloat,
) -> Result<(), TransactionError> {
    // NOTE: discarding transactions with negative amounts
    if amount > 0.0 {
        Ok(())
    } else {
        Err(TransactionError::AmountNotPositive { tx, amount })
    }
}

/// If the transaction is valid, return the transaction and a &mut to the associated account.
/// Otherwise, return an Err(TransactionError).
pub fn validate_deposit<'a, 't>(
    deposit: Deposit,
    accounts: &'a mut AccountsState,
    transactions: &'t TransactionsState,
) -> Result<(Deposit, impl UnlockedAccountFeatures + 'a), TransactionError> {
    check_for_duplicate_tx_id(deposit.tx_id, transactions)?;
    check_for_positive_amount(deposit.tx_id, deposit.amount)?;

    match accounts.get_mut_or_default(deposit.client_id) {
        AccountAccess::Unlocked(account) => Ok((deposit, account)),
        AccountAccess::Locked(_) => Err(TransactionError::AccountLocked {
            client: deposit.client_id,
            tx: deposit.tx_id,
        }),
    }
}

pub fn validate_withdrawal<'a, 't>(
    withdrawal: Withdrawal,
    accounts: &'a mut AccountsState,
    transactions: &'t TransactionsState,
) -> Result<(Withdrawal, impl UnlockedAccountFeatures + 'a), TransactionError> {
    check_for_duplicate_tx_id(withdrawal.tx_id, transactions)?;
    check_for_positive_amount(withdrawal.tx_id, withdrawal.amount)?;

    match accounts.get_mut(withdrawal.client_id) {
        // unlocked accounts can withdraw if they have enough funds
        Some(AccountAccess::Unlocked(account)) => {
            let view = account.view();
            if view.available >= withdrawal.amount {
                return Ok((withdrawal, account));
            } else {
                return Err(TransactionError::InsufficientFunds {
                    client: withdrawal.client_id,
                    tx: withdrawal.tx_id,
                    requested: withdrawal.amount,
                    available: view.available,
                });
            }
        }
        // Locked accounts cannot withdraw
        Some(AccountAccess::Locked(_)) => Err(TransactionError::AccountLocked {
            client: withdrawal.client_id,
            tx: withdrawal.tx_id,
        }),
        // New accounts cannot withdraw
        None => Err(TransactionError::InsufficientFunds {
            client: withdrawal.client_id,
            tx: withdrawal.tx_id,
            requested: withdrawal.amount,
            available: 0.0,
        }),
    }
}

fn validate_dispute_for_successful_tx<'a, 't, 'd, D: Disputable>(
    dispute: Dispute,
    disputed_tx: &'t D,
    accounts: &'a mut AccountsState,
    disputes: &'d DisputesState,
) -> Result<(&'t impl Disputable, Box<dyn BaseAccountFeatures + 'a>), TransactionError> {
    // NOTE: CHECK 3: dispute client_id must match disputed transaction client_id
    if dispute.client_id != disputed_tx.get_client_id() {
        return Err(TransactionError::ClientMismatch {
            tx: dispute.tx_id,
            tx_client: disputed_tx.get_client_id(),
            dispute_client: dispute.client_id,
        });
    }

    let tx_id = dispute.get_tx_id();
    let client_id = dispute.get_client_id();

    // NOTE: CHECK 4: Cannot dispute an actively disputed transaction
    if disputes.is_disputed(client_id, tx_id) {
        return Err(TransactionError::TxAlreadyDisputed {
            client: client_id,
            tx: tx_id,
        });
    }

    // NOTE: CHECK 5: Cannot dispute a settled transaction
    if disputes.is_settled(client_id, tx_id) {
        return Err(TransactionError::DisputeAlreadySettled {
            client: client_id,
            tx: tx_id,
        });
    }

    if let Some(access) = accounts.get_mut(client_id) {
        // Get access to the referenced account (don't need unlocked access here)
        let account = access.inner();
        return Ok((disputed_tx, account));
    } else {
        // This should never happen, but catch it just in case
        return Err(TransactionError::UnexpectedError(format!(
            "Disputed transaction {} refers to nonexistent client {}",
            tx_id, client_id
        )));
    }
}

/// Validate a dispute.
///
/// Assume:
/// 1.transaction exists
///
/// Need to check:
/// 1. transaction is of a disputable type
/// 2. transaction initially succeeded
/// 3. transaction refers to same client
/// 4. transaction is not actively disputed
/// 5. transaction is not already settled
pub fn validate_dispute<'a, 't, 'd>(
    dispute: Dispute,
    accounts: &'a mut AccountsState,
    transactions: &'t TransactionsState,
    disputes: &'d DisputesState,
) -> Result<(&'t impl Disputable, Box<dyn BaseAccountFeatures + 'a>), TransactionError> {
    // NOTE: disputes do not have their own transaction id, they refer to a deposit or withdrawal
    // NOTE: locked accounts are still allowed to dispute, just not deposit or withdraw

    // Get disputed transaction from log
    if let Some(disputed_tx_container) = transactions.get(dispute.client_id, dispute.tx_id) {
        match disputed_tx_container.try_get_disputable() {
            // Transaction is of a disputable type and initially succeeded
            Ok(Ok(disputed_tx)) => {
                validate_dispute_for_successful_tx(dispute, disputed_tx, accounts, disputes)
            }
            // Transaction is of a disputable type but initially failed
            Ok(Err(_)) => {
                // NOTE: CHECK 2: Cannot dispute a transaction that didn't succeed in the first place
                Err(TransactionError::DisputedTxFailed { tx: dispute.tx_id })
            }
            // CHECK 1: Transaction is not of a disputable type - its type is returned
            Err(tx_type) => Err(TransactionError::InvalidDispute {
                tx: dispute.tx_id,
                tx_type,
            }),
        }
    } else {
        Err(TransactionError::TxDoesNotExist {
            client: dispute.client_id,
            tx: dispute.tx_id,
        })
    }
}

fn validate_post_dispute_for_existing_tx<'a, 't, 'd, D: Disputable, P: PostDispute>(
    post: P,
    disputed_tx: &'t D,
    accounts: &'a mut AccountsState,
    disputes: &'d DisputesState,
) -> Result<(&'t impl Disputable, AccountAccess<'a>), TransactionError> {
    // NOTE: CHECK 1: client_id must match disputed transaction client_id
    if post.get_client_id() != disputed_tx.get_client_id() {
        return Err(TransactionError::ClientMismatch {
            tx: post.get_tx_id(),
            tx_client: disputed_tx.get_client_id(),
            dispute_client: post.get_client_id(),
        });
    }

    let tx_id = post.get_tx_id();
    let client_id = post.get_client_id();

    let disputed = disputes.is_disputed(client_id, tx_id);
        // NOTE: CHECK 2: Cannot dispute an actively disputed transaction
        if !disputed {
            return Err(TransactionError::TxNotDisputed {
                client: client_id,
                tx: tx_id,
            });
        }

    if let Some(access) = accounts.get_mut(client_id) {
        return Ok((disputed_tx, access));
    } else {
        // This should never happen, but catch it just in case
        return Err(TransactionError::UnexpectedError(format!(
            "Disputed transaction {} refers to nonexistent client {}",
            tx_id, client_id
        )));
    }
}

/// Validate a reolve or chargeback.
///
/// Assume:
/// 1.transaction exists
///
/// Need to check:
/// 1. transaction refers to same client
/// 2. transaction is actively disputed
pub fn validate_post_dispute<'a, 't, 'd, T: PostDispute>(
    post: T,
    accounts: &'a mut AccountsState,
    transactions: &'t TransactionsState,
    disputes: &'d DisputesState,
) -> Result<(&'t impl Disputable, AccountAccess<'a>), TransactionError> {
    // NOTE: disputes and resolvess do not have their own transaction id,
    // they refer to a deposit or withdrawal
    // NOTE: locked accounts are still allowed to dispute and resolve,
    // just not deposit or withdraw

    let client_id = post.get_client_id();
    let tx_id = post.get_tx_id();

    // Get disputed transaction from log
    if let Some(disputed_tx_container) = transactions.get(client_id, tx_id) {
        if let Ok(Ok(disputed_tx)) = disputed_tx_container.try_get_disputable() {
            validate_post_dispute_for_existing_tx(post, disputed_tx, accounts, disputes)
        } else {
            // NOTE: Actively disputed transaction should have already been validated
            Err(TransactionError::UnexpectedError(format!(
                "Cannot retrieve actively disputed transaction {}",
                post.get_tx_id()
            )))
        }
    } else {
        Err(TransactionError::TxDoesNotExist {
            client: client_id,
            tx: tx_id,
        })
    }
}
