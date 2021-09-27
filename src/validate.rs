use crate::types::State;
use crate::types::TransactionId;
use crate::types::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};
use crate::types::{TransactionContainer, TransactionError};

fn check_for_duplicate_tx_id(tx_id: TransactionId, state: &State) -> Result<(), TransactionError> {
    // NOTE: discarding duplicate transactions
    // TODO: Efficiently record duplicate transactions?

    if let Some(_tx) = state.transactions.get(&tx_id) {
        // Duplicate transactions are a bad sign
        Err(TransactionError::DuplicateTxId)
    } else {
        Ok(())
    }
}

pub fn validate_deposit(deposit: &Deposit, state: &State) -> Result<(), TransactionError> {
    check_for_duplicate_tx_id(deposit.tx_id, state)?;

    if let Some(account) = state.accounts.get(&deposit.client_id) {
        if account.locked {
            // Locked accounts cannot deposit
            return Err(TransactionError::AccountLocked);
        }
    }

    // New and unlocked accounts can deposit
    Ok(())
}

pub fn validate_withdrawal(withdrawal: &Withdrawal, state: &State) -> Result<(), TransactionError> {
    check_for_duplicate_tx_id(withdrawal.tx_id, state)?;

    if let Some(account) = state.accounts.get(&withdrawal.client_id) {
        if account.locked {
            // Locked accounts cannot withdraw
            return Err(TransactionError::AccountLocked);
        } else {
            // unlocked accounts can withdraw if they have enough funds
            if account.available >= withdrawal.amount {
                Ok(())
            } else {
                return Err(TransactionError::InsufficientFunds {
                    required: withdrawal.amount,
                    actual: account.available,
                });
            }
        }
    } else {
        // New accounts cannot withdraw
        // TODO: This would be a weird error for a 0-amount withdrawal
        return Err(TransactionError::InsufficientFunds {
            required: withdrawal.amount,
            actual: 0.0,
        });
    }
}

pub fn validate_dispute(dispute: &Dispute, state: &State) -> Result<(), TransactionError> {
    // NOTE: disputes do not have their own transaction id, they refer to a deposit or withdrawal
    // NOTE: locked accounts are still allowed to dispute, just not deposit or withdraw

    // NOTE: Cannot dispute an actively disputed transaction
    if state.active_disputes.contains(&dispute.tx_id) {
        return Err(TransactionError::TxAlreadyDisputed);
    }

    // Get disputed transaction from log
    if let Some(disputed_transaction) = state.transactions.get(&dispute.tx_id) {
        // NOTE: Only deposits may be disputed
        if let TransactionContainer::Deposit(_) = disputed_transaction {
            // TODO: Verify that disputed deposit actually succeeded
            Ok(())
        } else {
            Err(TransactionError::InvalidDispute)
        }
    } else {
        Err(TransactionError::TxDoesNotExist)
    }
}

pub fn validate_resolve(resolve: &Resolve, state: &State) -> Result<(), TransactionError> {
    // NOTE: resolves do not have their own transaction id, they refer to a deposit or withdrawal
    // NOTE: locked accounts are still allowed to resolve, just not deposit or withdraw

    // NOTE: Cannot resolve an undisputed transaction
    if state.active_disputes.contains(&resolve.tx_id) {
        Ok(())
    } else {
        Err(TransactionError::TxNotDisputed)
    }
}

pub fn validate_chargeback(chargeback: &Chargeback, state: &State) -> Result<(), TransactionError> {
    // NOTE: chargebacks do not have their own transaction id, they refer to a deposit or withdrawal
    // NOTE: locked accounts are still allowed to chargeback, just not deposit or withdraw

    // NOTE: Cannot chargeback an undisputed transaction
    if state.active_disputes.contains(&chargeback.tx_id) {
        Ok(())
    } else {
        Err(TransactionError::TxNotDisputed)
    }
}
