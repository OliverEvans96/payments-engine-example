use crate::currency::CurrencyFloat;
use crate::types::State;
use crate::types::TransactionId;
use crate::types::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};
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
    if amount > 0.0 {
        Ok(())
    } else {
        Err(TransactionError::AmountNotPositive { tx, amount })
    }
}

pub fn validate_deposit(deposit: &Deposit, state: &State) -> Result<(), TransactionError> {
    check_for_duplicate_tx_id(deposit.tx_id, state)?;
    check_for_positive_amount(deposit.tx_id, deposit.amount)?;

    if let Some(account) = state.accounts.get(&deposit.client_id) {
        if account.locked {
            // Locked accounts cannot deposit
            return Err(TransactionError::AccountLocked {
                client: deposit.client_id,
                tx: deposit.tx_id,
            });
        }
    }

    // New and unlocked accounts can deposit
    Ok(())
}

pub fn validate_withdrawal(withdrawal: &Withdrawal, state: &State) -> Result<(), TransactionError> {
    check_for_duplicate_tx_id(withdrawal.tx_id, state)?;
    check_for_positive_amount(withdrawal.tx_id, withdrawal.amount)?;

    if let Some(account) = state.accounts.get(&withdrawal.client_id) {
        if account.locked {
            // Locked accounts cannot withdraw
            return Err(TransactionError::AccountLocked {
                client: withdrawal.client_id,
                tx: withdrawal.tx_id,
            });
        } else {
            // unlocked accounts can withdraw if they have enough funds
            if account.available >= withdrawal.amount {
                Ok(())
            } else {
                return Err(TransactionError::InsufficientFunds {
                    client: withdrawal.client_id,
                    tx: withdrawal.tx_id,
                    requested: withdrawal.amount,
                    available: account.available,
                });
            }
        }
    } else {
        // New accounts cannot withdraw
        return Err(TransactionError::InsufficientFunds {
            client: withdrawal.client_id,
            tx: withdrawal.tx_id,
            requested: withdrawal.amount,
            available: 0.0,
        });
    }
}

pub fn validate_dispute(dispute: &Dispute, state: &State) -> Result<(), TransactionError> {
    // NOTE: disputes do not have their own transaction id, they refer to a deposit or withdrawal
    // NOTE: locked accounts are still allowed to dispute, just not deposit or withdraw

    // NOTE: Cannot dispute an actively disputed transaction
    if state.active_disputes.contains(&dispute.tx_id) {
        return Err(TransactionError::TxAlreadyDisputed {
            client: dispute.client_id,
            tx: dispute.tx_id,
        });
    }

    // TODO: Check that dispute client matches disputed transaction client_id

    // Get disputed transaction from log
    if let Some(disputed_transaction) = state.transactions.get(&dispute.tx_id) {
        // NOTE: Only deposits may be disputed
        match disputed_transaction {
            TransactionContainer::Deposit(_) => {
                // TODO: Verify that disputed deposit actually succeeded
                Ok(())
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

pub fn validate_resolve(resolve: &Resolve, state: &State) -> Result<(), TransactionError> {
    // NOTE: resolves do not have their own transaction id, they refer to a deposit or withdrawal
    // NOTE: locked accounts are still allowed to resolve, just not deposit or withdraw

    // TODO: Check that transaction exists?

    // NOTE: Cannot resolve an undisputed transaction
    if state.active_disputes.contains(&resolve.tx_id) {
        Ok(())
    } else {
        Err(TransactionError::TxNotDisputed {
            client: resolve.client_id,
            tx: resolve.tx_id,
        })
    }
}

pub fn validate_chargeback(chargeback: &Chargeback, state: &State) -> Result<(), TransactionError> {
    // NOTE: chargebacks do not have their own transaction id, they refer to a deposit or withdrawal
    // NOTE: locked accounts are still allowed to chargeback, just not deposit or withdraw

    // TODO: Check that transaction exists?

    // NOTE: Cannot chargeback an undisputed transaction
    if state.active_disputes.contains(&chargeback.tx_id) {
        Ok(())
    } else {
        Err(TransactionError::TxNotDisputed {
            client: chargeback.client_id,
            tx: chargeback.tx_id,
        })
    }
}
