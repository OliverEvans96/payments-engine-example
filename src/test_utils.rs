use std::collections::HashMap;

use crate::handlers::handle_transaction;
use crate::state::{AccountsState, State};
use crate::types::{Account, ClientId, TransactionError, TransactionRecord};

/// Given an initial state and a set of transactions,
/// test that the final account states and generated errors
/// both match their expected values.
pub fn run_test_scenario(
    initial_state: State,
    transactions: Vec<TransactionRecord>,
    final_accounts: HashMap<ClientId, Account>,
    expected_errors: Vec<TransactionError>,
) {
    let mut state = initial_state;
    let mut actual_errors = Vec::new();

    for transaction in transactions {
        if let Err(err) = handle_transaction(transaction, &mut state) {
            actual_errors.push(err);
        }
    }

    let final_accounts_state: AccountsState = final_accounts.into();

    assert_eq!(final_accounts_state, state.accounts);
    assert_eq!(expected_errors, actual_errors);
}
