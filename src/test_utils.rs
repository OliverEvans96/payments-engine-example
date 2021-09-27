use std::collections::HashMap;

use crate::handlers::handle_transaction;
use crate::types::{Account, ClientId, State, TransactionRecord};

pub fn run_test_scenario(
    initial_state: State,
    transactions: Vec<TransactionRecord>,
    final_accounts: HashMap<ClientId, Account>,
) {
    let mut state = initial_state;
    for transaction in transactions {
        handle_transaction(transaction, &mut state);
    }
    assert_eq!(state.accounts, final_accounts);
}
