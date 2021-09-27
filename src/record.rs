use crate::types::TransactionId;
use crate::types::{Deposit, Withdrawal};
use crate::types::{State, TransactionContainer, TransactionError};

/// Record a deposit, either valid or invalid, in the transaction log
pub fn record_deposit_result(
    tx_id: TransactionId,
    result: Result<Deposit, TransactionError>,
    state: &mut State,
) {
    state
        .transactions
        .entry(tx_id)
        .or_insert(TransactionContainer::Deposit(result));
}

/// Record a withdrawal, either valid or invalid, in the transaction log
pub fn record_withdrawal_result(
    tx_id: TransactionId,
    result: Result<Withdrawal, TransactionError>,
    state: &mut State,
) {
    state
        .transactions
        .entry(tx_id)
        .or_insert(TransactionContainer::Withdrawal(result));
}

/// Mark a transaction as actively disputed
pub fn dispute_transaction(tx_id: TransactionId, state: &mut State) {
    let success = state.active_disputes.insert(tx_id);
    if !success {
        log::warn!("Transaction {} has been doubly disputed.", tx_id);
    }
}

/// Mark a transaction as no longer actively disputed
pub fn undispute_transaction(tx_id: TransactionId, state: &mut State) {
    let success = state.active_disputes.remove(&tx_id);
    if !success {
        log::warn!(
            "Transaction {} has been undisputed, but wasn't previously disputed.",
            tx_id
        );
    }
}
