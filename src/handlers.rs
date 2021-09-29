use crate::account::{AccountAccess, BaseAccountFeatures, UnlockedAccountFeatures};
use crate::currency::round_currency;
use crate::state::State;
use crate::types::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};
use crate::types::{TransactionContainer, TransactionError, TransactionRecord, TransactionType};
use crate::validate;

fn handle_deposit(deposit: Deposit, state: &mut State) -> Result<(), TransactionError> {
    log::trace!("Handling {:?}", deposit);
    let tx_id = deposit.tx_id;
    match validate::validate_deposit(deposit, &mut state.accounts, &state.transactions) {
        Ok((valid_deposit, mut account)) => {
            account.modify_balances_for_deposit(&valid_deposit);
            state
                .transactions
                .insert(tx_id, TransactionContainer::Deposit(Ok(valid_deposit)));
            Ok(())
        }
        Err(err) => {
            state
                .transactions
                .insert(tx_id, TransactionContainer::Deposit(Err(err.clone())));
            Err(err)
        }
    }
}

fn handle_withdrawal(withdrawal: Withdrawal, state: &mut State) -> Result<(), TransactionError> {
    log::trace!("Handling {:?}", withdrawal);
    let tx_id = withdrawal.tx_id;
    match validate::validate_withdrawal(withdrawal, &mut state.accounts, &state.transactions) {
        Ok((valid_withdrawal, mut account)) => {
            account.modify_balances_for_withdrawal(&valid_withdrawal);
            state.transactions.insert(
                tx_id,
                TransactionContainer::Withdrawal(Ok(valid_withdrawal)),
            );
            Ok(())
        }
        Err(err) => {
            state
                .transactions
                .insert(tx_id, TransactionContainer::Withdrawal(Err(err.clone())));
            Err(err)
        }
    }
}

fn handle_dispute(dispute: Dispute, state: &mut State) -> Result<(), TransactionError> {
    log::trace!("Handling {:?}", dispute);
    let tx_id = dispute.tx_id;
    match validate::validate_dispute(
        dispute,
        &mut state.accounts,
        &state.transactions,
        &state.disputes,
    ) {
        Ok((disputed_tx, mut account)) => {
            account.modify_balances_for_dispute(disputed_tx);
            state.disputes.dispute_tx(tx_id);
            Ok(())
        }
        Err(err) => Err(err),
    }
}

fn handle_resolve(resolve: Resolve, state: &mut State) -> Result<(), TransactionError> {
    log::trace!("Handling {:?}", resolve);
    let tx_id = resolve.tx_id;
    match validate::validate_post_dispute(
        resolve,
        &mut state.accounts,
        &state.transactions,
        &state.disputes,
    ) {
        Ok((disputed_tx, mut access)) => {
            access.modify_balances_for_resolve(disputed_tx);
            state.disputes.undispute_tx(tx_id);
            Ok(())
        }
        Err(err) => Err(err),
    }
}

fn handle_chargeback(chargeback: Chargeback, state: &mut State) -> Result<(), TransactionError> {
    log::trace!("Handling {:?}", chargeback);
    let tx_id = chargeback.tx_id;
    match validate::validate_post_dispute(
        chargeback,
        &mut state.accounts,
        &state.transactions,
        &state.disputes,
    ) {
        Ok((disputed_tx, mut access)) => {
            access.modify_balances_for_chargeback(disputed_tx);
            if let AccountAccess::Unlocked(mut account) = access {
                account.lock();
            }
            state.disputes.undispute_tx(tx_id);
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
