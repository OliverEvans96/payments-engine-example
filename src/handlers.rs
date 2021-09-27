use crate::record;
use crate::types::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};
use crate::types::{State, TransactionError, TransactionRecord, TransactionType};
use crate::validate;

fn handle_deposit(deposit: Deposit, state: &mut State) -> Result<(), TransactionError> {
    validate::validate_deposit(&deposit, state)?;
    record::record_deposit(deposit, state);
    Ok(())
}

fn handle_withdrawal(withdrawal: Withdrawal, state: &mut State) -> Result<(), TransactionError> {
    validate::validate_withdrawal(&withdrawal, state)?;
    record::record_withdrawal(withdrawal, state);
    Ok(())
}

fn handle_dispute(dispute: Dispute, state: &mut State) -> Result<(), TransactionError> {
    validate::validate_dispute(&dispute, state)?;
    record::record_dispute(dispute, state);
    Ok(())
}

fn handle_resolve(resolve: Resolve, state: &mut State) -> Result<(), TransactionError> {
    validate::validate_resolve(&resolve, state)?;
    record::record_resolve(resolve, state);
    Ok(())
}

fn handle_chargeback(chargeback: Chargeback, state: &mut State) -> Result<(), TransactionError> {
    validate::validate_chargeback(&chargeback, state)?;
    record::record_chargeback(chargeback, state);
    Ok(())
}

pub fn handle_transaction(
    record: TransactionRecord,
    state: &mut State,
) -> Result<(), TransactionError> {
    match record {
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id,
            tx_id,
            amount: Some(amount),
        } => {
            let deposit = Deposit {
                client_id,
                tx_id,
                amount,
            };
            handle_deposit(deposit, state)
        }
        TransactionRecord {
            transaction_type: TransactionType::Withdrawal,
            client_id,
            tx_id,
            amount: Some(amount),
        } => {
            let withdrawal = Withdrawal {
                client_id,
                tx_id,
                amount,
            };
            handle_withdrawal(withdrawal, state)
        }
        TransactionRecord {
            transaction_type: TransactionType::Dispute,
            client_id,
            tx_id,
            amount: None,
        } => {
            let dispute = Dispute { client_id, tx_id };
            handle_dispute(dispute, state)
        }
        TransactionRecord {
            transaction_type: TransactionType::Resolve,
            client_id,
            tx_id,
            amount: None,
        } => {
            let resolve = Resolve { client_id, tx_id };
            handle_resolve(resolve, state)
        }
        TransactionRecord {
            transaction_type: TransactionType::Chargeback,
            client_id,
            tx_id,
            amount: None,
        } => {
            let chargeback = Chargeback { client_id, tx_id };
            handle_chargeback(chargeback, state)
        }
        _ => Err(TransactionError::ImproperTransaction(record)),
    }
}
