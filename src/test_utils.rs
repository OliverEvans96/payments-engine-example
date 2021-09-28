use std::collections::HashMap;

use crate::handlers::handle_transaction;
use crate::state::{State,AccountsState};
use crate::types::{Account, ClientId, TransactionError, TransactionRecord};

pub fn run_test_scenario(
    initial_state: State,
    transactions: Vec<TransactionRecord>,
    final_accounts: HashMap<ClientId, Account>,
) -> Result<(), TransactionError> {
    let mut state = initial_state;
    for transaction in transactions {
        handle_transaction(transaction, &mut state)?;
    }

    let final_accounts_state: AccountsState = final_accounts.into();

    assert_eq!(final_accounts_state, state.accounts);

    Ok(())
}
