use crate::account;
use crate::currency::round_currency;
use crate::record;
use crate::types::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};
use crate::types::{State, TransactionError, TransactionRecord, TransactionType};
use crate::validate;

fn handle_deposit(deposit: Deposit, state: &mut State) -> Result<(), TransactionError> {
    let tx_id = deposit.tx_id;
    match validate::validate_deposit(deposit, state) {
        Ok((valid_deposit, account)) => {
            account::modify_balances_for_deposit(&valid_deposit, account);
            record::record_deposit_result(tx_id, Ok(valid_deposit), state);
            Ok(())
        }
        Err(err) => {
            record::record_deposit_result(tx_id, Err(err.clone()), state);
            Err(err)
        }
    }
}

fn handle_withdrawal(withdrawal: Withdrawal, state: &mut State) -> Result<(), TransactionError> {
    let tx_id = withdrawal.tx_id;
    match validate::validate_withdrawal(withdrawal, state) {
        Ok((valid_withdrawal, account)) => {
            account::modify_balances_for_withdrawal(&valid_withdrawal, account);
            record::record_withdrawal_result(tx_id, Ok(valid_withdrawal), state);
            Ok(())
        }
        Err(err) => {
            record::record_withdrawal_result(tx_id, Err(err.clone()), state);
            Err(err)
        }
    }
}

fn handle_dispute(dispute: Dispute, state: &mut State) -> Result<(), TransactionError> {
    let tx_id = dispute.tx_id;
    match validate::validate_dispute(dispute, state) {
        Ok((disputed_deposit, account)) => {
            account::modify_balances_for_dispute(disputed_deposit, account);
            record::dispute_transaction(tx_id, state);
            Ok(())
        }
        Err(err) => Err(err),
    }
}

fn handle_resolve(resolve: Resolve, state: &mut State) -> Result<(), TransactionError> {
    let tx_id = resolve.tx_id;
    match validate::validate_post_dispute(resolve, state) {
        Ok((disputed_deposit, account)) => {
            account::modify_balances_for_resolve(disputed_deposit, account);
            record::undispute_transaction(tx_id, state);
            Ok(())
        }
        Err(err) => Err(err),
    }
}

fn handle_chargeback(chargeback: Chargeback, state: &mut State) -> Result<(), TransactionError> {
    let tx_id = chargeback.tx_id;
    match validate::validate_post_dispute(chargeback, state) {
        Ok((disputed_deposit, account)) => {
            account::modify_balances_for_chargeback(disputed_deposit, account);
            account::lock_account(account);
            record::undispute_transaction(tx_id, state);
            Ok(())
        }
        Err(err) => Err(err),
    }
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
                amount: round_currency(amount),
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
                amount: round_currency(amount),
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
