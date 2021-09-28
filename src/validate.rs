use crate::account::AccountAccess;
use crate::account::{BaseAccountFeatures, UnlockedAccountFeatures};
use crate::currency::CurrencyFloat;
use crate::state::{AccountsState, DisputesState, TransactionsState};
use crate::types::{Deposit, Dispute, PostDispute, Withdrawal};
use crate::types::{Disputable, Transaction, TransactionId};
use crate::types::{TransactionContainer, TransactionError};

fn check_for_duplicate_tx_id(
    tx_id: TransactionId,
    transactions: &TransactionsState,
) -> Result<(), TransactionError> {
    // NOTE: discarding duplicate transactions
    // TODO: Efficiently record duplicate transactions?
    if let Some(_) = transactions.get(tx_id) {
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

pub fn validate_dispute<'a, 't, 'd>(
    dispute: Dispute,
    accounts: &'a mut AccountsState,
    transactions: &'t TransactionsState,
    disputes: &'d DisputesState,
) -> Result<(&'t impl Disputable, Box<dyn BaseAccountFeatures + 'a>), TransactionError> {
    // NOTE: disputes do not have their own transaction id, they refer to a deposit or withdrawal
    // NOTE: locked accounts are still allowed to dispute, just not deposit or withdraw

    // NOTE: Cannot dispute an actively disputed transaction
    if disputes.is_disputed(dispute.tx_id) {
        return Err(TransactionError::TxAlreadyDisputed {
            client: dispute.client_id,
            tx: dispute.tx_id,
        });
    }

    // Get disputed transaction from log
    if let Some(disputed_tx_container) = transactions.get(dispute.tx_id) {
        match disputed_tx_container.try_get_disputable() {
            // Transaction is disputable and initially succeeded
            Ok(Ok(disputed_tx)) => {
                let client_id = disputed_tx.get_client_id();
                // NOTE: Only deposits may be disputed
                // TransactionContainer::Deposit(Ok(disputed_tx)) => {
                // NOTE: dispute client_id must match disputed transaction client_id
                if client_id != dispute.client_id {
                    Err(TransactionError::DisputeClientMismatch {
                        tx: dispute.tx_id,
                        tx_client: client_id,
                        dispute_client: dispute.client_id,
                    })
                } else {
                    // Get access to the referenced account (don't need unlocked access here)
                    match accounts.get_mut(dispute.client_id) {
                        Some(access) => {
                            let account = access.inner();
                            Ok((disputed_tx, account))
                        }
                        None => {
                            // This should never happen, but catch it just in case
                            Err(TransactionError::UnexpectedError(format!(
                                "Disputed transaction {} refers to nonexistent client {}",
                                dispute.tx_id, dispute.client_id
                            )))
                        }
                    }
                }
            }
            // Transaction is disputable but initially failed
            Ok(Err(_)) => {
                // NOTE: Cannot dispute a transaction that didn't succeed in the first place
                Err(TransactionError::DisputedTxFailed { tx: dispute.tx_id })
            }
            // Transaction is not disputable - its type is returned
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

/// Validation is the same for resolves and chargebacks
pub fn validate_post_dispute<'a, 't, 'd, T: PostDispute>(
    tx: T,
    accounts: &'a mut AccountsState,
    transactions: &'t TransactionsState,
    disputes: &'d DisputesState,
) -> Result<(&'t Deposit, AccountAccess<'a>), TransactionError> {
    // NOTE: disputes and resolvess do not have their own transaction id,
    // they refer to a deposit or withdrawal
    // NOTE: locked accounts are still allowed to dispute and resolve,
    // just not deposit or withdraw

    let tx_id = tx.get_tx_id();
    let client_id = tx.get_client_id();

    // NOTE: Cannot dispute an undisputed transaction
    if !disputes.is_disputed(tx_id) {
        return Err(TransactionError::TxNotDisputed {
            client: client_id,
            tx: tx_id,
        });
    }

    // Get disputed transaction from log
    if let Some(TransactionContainer::Deposit(Ok(disputed_tx))) = transactions.get(tx_id) {
        // NOTE: client_id must match disputed transaction client_id
        if disputed_tx.client_id != client_id {
            Err(TransactionError::DisputeClientMismatch {
                tx: tx_id,
                tx_client: disputed_tx.client_id,
                dispute_client: client_id,
            })
        } else {
            match accounts.get_mut(client_id) {
                Some(access) => Ok((disputed_tx, access)),
                None => {
                    // This should never happen, but catch it just in case
                    Err(TransactionError::UnexpectedError(format!(
                        "Disputed transaction {} refers to nonexistent client {}",
                        tx_id, client_id
                    )))
                }
            }
        }
    } else {
        // NOTE: Actively disputed transaction should have already been validated
        Err(TransactionError::UnexpectedError(format!(
            "Cannot retrieve actively disputed transaction {}",
            tx_id
        )))
    }
}
