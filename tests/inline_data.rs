use std::collections::HashMap;

use payments_engine_example::test_utils::run_test_scenario;
use payments_engine_example::types::{
    Account, State, TransactionError, TransactionRecord, TransactionType,
};

#[test]
fn deposit_new_account() -> Result<(), TransactionError> {
    let initial_state = State::new();

    let transactions = vec![TransactionRecord {
        transaction_type: TransactionType::Deposit,
        client_id: 1,
        tx_id: 1,
        amount: Some(5.0),
    }];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 5.0,
            held: 0.0,
            locked: false,
        },
    );

    run_test_scenario(initial_state, transactions, final_accounts)?;

    Ok(())
}

#[test]
fn deposit_existing_account() -> Result<(), TransactionError> {
    let mut initial_state = State::new();
    initial_state.accounts.insert(
        1,
        Account {
            available: 7.0,
            held: 0.0,
            locked: false,
        },
    );

    let transactions = vec![TransactionRecord {
        transaction_type: TransactionType::Deposit,
        client_id: 1,
        tx_id: 1,
        amount: Some(5.0),
    }];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 12.0,
            held: 0.0,
            locked: false,
        },
    );

    run_test_scenario(initial_state, transactions, final_accounts)?;

    Ok(())
}
