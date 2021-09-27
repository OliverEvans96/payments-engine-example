use crate::balances;
use crate::types::{Account, State, TransactionContainer};
use crate::types::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};

// NOTE: Assuming transaction has already been validated
pub fn record_deposit(deposit: Deposit, state: &mut State) {
    // Update account
    state
        .accounts
        .entry(deposit.client_id)
        // Modify account if it's present
        .and_modify(|account| balances::modify_balances_for_deposit(&deposit, account))
        // Otherwise, lazily create new account
        .or_insert_with(|| Account {
            available: deposit.amount,
            ..Default::default()
        });

    // Log transaction
    state
        .transactions
        .entry(deposit.tx_id)
        .or_insert(TransactionContainer::Deposit(Ok(deposit)));
}

// NOTE: Assuming transaction has already been validated
pub fn record_withdrawal(withdrawal: Withdrawal, state: &mut State) {
    // Since withdrawing from an account with no existing balance is invalid,
    // we can assume that account already exists (and unwrap the option)
    let account = state.accounts.get_mut(&withdrawal.client_id).unwrap();

    balances::modify_balances_for_withdrawal(&withdrawal, account);

    // Log transaction
    state
        .transactions
        .entry(withdrawal.tx_id)
        .or_insert(TransactionContainer::Withdrawal(Ok(withdrawal)));
}

// NOTE: Assuming dispute has already been validated
pub fn record_dispute(dispute: Dispute, state: &mut State) {
    if let Some(TransactionContainer::Deposit(Ok(disputed_deposit))) =
        state.transactions.get(&dispute.tx_id)
    {
        // Get associated account
        if let Some(account) = state.accounts.get_mut(&dispute.client_id) {
            balances::modify_balances_for_dispute(disputed_deposit, account);

            // Mark the transaction as actively disputed
            let success = state.active_disputes.insert(dispute.tx_id);

            if !success {
                log::warn!("Transaction {} has been doubly disputed", dispute.tx_id);
            }
        } else {
            log::warn!(
                "Attempted to record dispute for nonexistent account - did you forget to validate?"
            );
        }
    } else {
        log::warn!("Attempted to record invalid dispute - did you forget to validate?");
    }
}

pub fn record_resolve(resolve: Resolve, state: &mut State) {
    if let Some(TransactionContainer::Deposit(Ok(disputed_deposit))) =
        state.transactions.get(&resolve.tx_id)
    {
        // Get associated account
        if let Some(account) = state.accounts.get_mut(&resolve.client_id) {
            balances::modify_balances_for_resolve(disputed_deposit, account);

            // Mark the transaction as no longer disputed
            let success = state.active_disputes.remove(&resolve.tx_id);

            if !success {
                // TODO: Avoid this
                log::warn!(
                    "Transaction {} has been resolved, but it wasn't disputed",
                    resolve.tx_id
                );
            }
        } else {
            log::warn!(
                "Attempted to record resolve for nonexistent account - did you forget to validate?"
            );
        }
    } else {
        log::warn!("Attempted to record invalid resolve - did you forget to validate?");
    }
}

pub fn record_chargeback(chargeback: Chargeback, state: &mut State) {
    if let Some(TransactionContainer::Deposit(Ok(disputed_deposit))) =
        state.transactions.get(&chargeback.tx_id)
    {
        // Get associated account
        if let Some(account) = state.accounts.get_mut(&chargeback.client_id) {
            balances::modify_balances_for_chargeback(disputed_deposit, account);

            // Mark the transaction as no longer disputed
            let success = state.active_disputes.remove(&chargeback.tx_id);

            // Lock the account
            account.locked = true;

            if !success {
                // TODO: Avoid this
                log::warn!(
                    "Transaction {} has been charged back, but it wasn't disputed",
                    chargeback.tx_id
                );
            }
        } else {
            log::warn!(
            "Attempted to record chargeback for nonexistent account - did you forget to validate?"
        );
        }
    } else {
        log::warn!("Attempted to record invalid chargeback - did you forget to validate?");
    }
}
