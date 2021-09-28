use crate::account::{AccountAccess, UnlockedAccount};
use crate::account::{BaseAccountFeatures, UnlockedAccountFeatures};
use crate::currency::CurrencyFloat;
use crate::state::State;
use crate::types::Account;
use crate::types::TransactionId;
use crate::types::{Deposit, Dispute, PostDispute, Withdrawal};
use crate::types::{TransactionContainer, TransactionError};

fn check_for_duplicate_tx_id(tx_id: TransactionId, state: &State) -> Result<(), TransactionError> {
    // NOTE: discarding duplicate transactions
    // TODO: Efficiently record duplicate transactions?

    if let Some(_tx) = state.transactions.get(&tx_id) {
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
pub fn validate_deposit<'a>(
    deposit: Deposit,
    state: &'a mut State,
) -> Result<(Deposit, impl UnlockedAccountFeatures + 'a), TransactionError> {
    check_for_duplicate_tx_id(deposit.tx_id, state)?;
    check_for_positive_amount(deposit.tx_id, deposit.amount)?;

    match state.get_mut_account_or_default(deposit.client_id) {
        AccountAccess::Unlocked(account) => Ok((deposit, account)),
        AccountAccess::Locked(_) => Err(TransactionError::AccountLocked {
            client: deposit.client_id,
            tx: deposit.tx_id,
        }),
    }
}

pub fn validate_withdrawal<'a>(
    withdrawal: Withdrawal,
    state: &'a mut State,
) -> Result<(Withdrawal, impl UnlockedAccountFeatures + 'a), TransactionError> {
    check_for_duplicate_tx_id(withdrawal.tx_id, state)?;
    check_for_positive_amount(withdrawal.tx_id, withdrawal.amount)?;

    match state.get_mut_account(withdrawal.client_id) {
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

pub fn validate_dispute<'a>(
    dispute: Dispute,
    state: &'a mut State,
) -> Result<(&Deposit, impl BaseAccountFeatures + 'a), TransactionError> {
    // NOTE: disputes do not have their own transaction id, they refer to a deposit or withdrawal
    // NOTE: locked accounts are still allowed to dispute, just not deposit or withdraw

    // NOTE: Cannot dispute an actively disputed transaction
    if state.active_disputes.contains(&dispute.tx_id) {
        return Err(TransactionError::TxAlreadyDisputed {
            client: dispute.client_id,
            tx: dispute.tx_id,
        });
    }

    // Get disputed transaction from log
    if let Some(disputed_transaction) = state.transactions.get(&dispute.tx_id) {
        match disputed_transaction {
            // NOTE: Only deposits may be disputed
            TransactionContainer::Deposit(Ok(disputed_deposit)) => {
                // NOTE: dispute client_id must match disputed transaction client_id
                if disputed_deposit.client_id != dispute.client_id {
                    Err(TransactionError::DisputeClientMismatch {
                        tx: dispute.tx_id,
                        tx_client: disputed_deposit.client_id,
                        dispute_client: dispute.client_id,
                    })
                } else {
                    // Get access to the referenced account (don't need unlocked access here)
                    match state.get_mut_account(dispute.client_id) {
                        Some(access) => {
                            let account = access.inner();
                            Ok((disputed_deposit, account))
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
            TransactionContainer::Deposit(Err(_)) => {
                // NOTE: Cannot dispute a transaction that didn't succeed in the first place
                Err(TransactionError::DisputedTxFailed { tx: dispute.tx_id })
            }
            other => Err(TransactionError::InvalidDispute {
                tx: dispute.tx_id,
                tx_type: other.tx_type(),
            }),
        }
    } else {
        Err(TransactionError::TxDoesNotExist {
            client: dispute.client_id,
            tx: dispute.tx_id,
        })
    }
}

pub fn validate_post_dispute<T: PostDispute>(
    t: T,
    state: &mut State,
) -> Result<(&Deposit, &mut Account), TransactionError> {
    // NOTE: ts do not have their own transaction id, they refer to a deposit or withdrawal
    // NOTE: locked accounts are still allowed to t, just not deposit or withdraw

    let tx_id = t.get_tx_id();
    let client_id = t.get_client_id();

    // NOTE: Cannot t an undisputed transaction
    if !state.active_disputes.contains(&tx_id) {
        return Err(TransactionError::TxNotDisputed {
            client: client_id,
            tx: tx_id,
        });
    }

    // Get disputed transaction from log
    if let Some(TransactionContainer::Deposit(Ok(disputed_deposit))) =
        state.transactions.get_mut(&tx_id)
    {
        // NOTE: t client_id must match disputed transaction client_id
        if disputed_deposit.client_id != client_id {
            Err(TransactionError::DisputeClientMismatch {
                tx: tx_id,
                tx_client: disputed_deposit.client_id,
                dispute_client: client_id,
            })
        } else {
            if let Some(account) = state.accounts.get_mut(&client_id) {
                Ok((disputed_deposit, account))
            } else {
                // This should never happen, but catch it just in case
                Err(TransactionError::UnexpectedError(format!(
                    "Disputed transaction {} refers to nonexistent client {}",
                    tx_id, client_id
                )))
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
