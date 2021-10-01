use std::collections::HashMap;

use payments_engine_example::state::State;
use payments_engine_example::test_utils::run_test_scenario;
use payments_engine_example::types::{
    Account, TransactionError, TransactionRecord, TransactionType,
};

#[test]
fn deposit_new_account() {
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

    let expected_errors = vec![];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn deposit_existing_account() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 1,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 2,
            amount: Some(5.0),
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 15.0,
            held: 0.0,
            locked: false,
        },
    );

    let expected_errors = vec![];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn deposit_no_amount() {
    let initial_state = State::new();

    let record = TransactionRecord {
        transaction_type: TransactionType::Deposit,
        client_id: 1,
        tx_id: 2,
        amount: None,
    };
    let transactions = vec![record.clone()];

    let final_accounts = HashMap::new();

    let expected_errors = vec![TransactionError::ImproperTransaction(record)];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn withdrawal_no_amount() {
    let initial_state = State::new();

    let record = TransactionRecord {
        transaction_type: TransactionType::Withdrawal,
        client_id: 1,
        tx_id: 2,
        amount: None,
    };
    let transactions = vec![record.clone()];

    let final_accounts = HashMap::new();

    let expected_errors = vec![TransactionError::ImproperTransaction(record)];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn dispute_has_amount() {
    let initial_state = State::new();

    let record = TransactionRecord {
        transaction_type: TransactionType::Dispute,
        client_id: 1,
        tx_id: 2,
        amount: Some(-92.0),
    };
    let transactions = vec![record.clone()];

    let final_accounts = HashMap::new();

    let expected_errors = vec![TransactionError::ImproperTransaction(record)];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn resolve_has_amount() {
    let initial_state = State::new();

    let record = TransactionRecord {
        transaction_type: TransactionType::Resolve,
        client_id: 1,
        tx_id: 2,
        amount: Some(-92.0),
    };
    let transactions = vec![record.clone()];

    let final_accounts = HashMap::new();

    let expected_errors = vec![TransactionError::ImproperTransaction(record)];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn chargeback_has_amount() {
    let initial_state = State::new();

    let record = TransactionRecord {
        transaction_type: TransactionType::Chargeback,
        client_id: 1,
        tx_id: 2,
        amount: Some(-92.0),
    };
    let transactions = vec![record.clone()];

    let final_accounts = HashMap::new();

    let expected_errors = vec![TransactionError::ImproperTransaction(record)];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn duplicate_tx_id_same_client() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 2,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 2,
            amount: Some(5.0),
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 10.0,
            held: 0.0,
            locked: false,
        },
    );

    let expected_errors = vec![TransactionError::DuplicateTxId { tx: 2 }];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn duplicate_tx_id_different_client() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 2,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 2,
            tx_id: 2,
            amount: Some(5.0),
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 10.0,
            held: 0.0,
            locked: false,
        },
    );

    let expected_errors = vec![TransactionError::DuplicateTxId { tx: 2 }];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn duplicate_tx_id_first_invalid() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 2,
            amount: Some(-10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 2,
            tx_id: 2,
            amount: Some(5.0),
        },
    ];

    let final_accounts = HashMap::new();

    let expected_errors = vec![
        TransactionError::AmountNotPositive {
            tx: 2,
            amount: -10.0,
        },
        TransactionError::DuplicateTxId { tx: 2 },
    ];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn unordered_tx_ids() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 7,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Withdrawal,
            client_id: 1,
            tx_id: 2,
            amount: Some(5.0),
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 5.0,
            held: 0.0,
            locked: false,
        },
    );

    let expected_errors = vec![];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn dispute_nonexistent_tx() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 7,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Dispute,
            client_id: 1,
            tx_id: 2,
            amount: None,
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 10.0,
            held: 0.0,
            locked: false,
        },
    );

    let expected_errors = vec![TransactionError::TxDoesNotExist { tx: 2, client: 1 }];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn resolve_nonexistent_tx() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 7,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Resolve,
            client_id: 1,
            tx_id: 2,
            amount: None,
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 10.0,
            held: 0.0,
            locked: false,
        },
    );

    let expected_errors = vec![TransactionError::TxDoesNotExist { tx: 2, client: 1 }];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn chargeback_nonexistent_tx() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 7,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Chargeback,
            client_id: 1,
            tx_id: 2,
            amount: None,
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 10.0,
            held: 0.0,
            locked: false,
        },
    );

    let expected_errors = vec![TransactionError::TxDoesNotExist { tx: 2, client: 1 }];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
#[ignore]
fn dispute_client_mismatch() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 7,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Dispute,
            client_id: 2,
            tx_id: 7,
            amount: None,
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 10.0,
            held: 0.0,
            locked: false,
        },
    );

    let expected_errors = vec![TransactionError::ClientMismatch {
        tx: 2,
        dispute_client: 2,
        tx_client: 1,
    }];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
#[ignore]
fn resolve_client_mismatch() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 7,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Dispute,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
        TransactionRecord {
            transaction_type: TransactionType::Resolve,
            client_id: 2,
            tx_id: 7,
            amount: None,
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 0.0,
            held: 10.0,
            locked: false,
        },
    );

    let expected_errors = vec![TransactionError::ClientMismatch {
        tx: 2,
        dispute_client: 2,
        tx_client: 1,
    }];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn resolve_undisputed_tx() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 7,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Resolve,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 10.0,
            held: 0.0,
            locked: false,
        },
    );

    let expected_errors = vec![TransactionError::TxNotDisputed { client: 1, tx: 7 }];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn double_dispute() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 7,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Dispute,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
        TransactionRecord {
            transaction_type: TransactionType::Dispute,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 0.0,
            held: 10.0,
            locked: false,
        },
    );

    let expected_errors = vec![TransactionError::TxAlreadyDisputed { client: 1, tx: 7 }];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn dispute_after_resolve() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 7,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Dispute,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
        TransactionRecord {
            transaction_type: TransactionType::Resolve,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
        TransactionRecord {
            transaction_type: TransactionType::Dispute,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 10.0,
            held: 0.0,
            locked: false,
        },
    );

    let expected_errors = vec![TransactionError::DisputeAlreadySettled { client: 1, tx: 7 }];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn dispute_after_chargeback() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 7,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Dispute,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
        TransactionRecord {
            transaction_type: TransactionType::Chargeback,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
        TransactionRecord {
            transaction_type: TransactionType::Dispute,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 0.0,
            held: 0.0,
            locked: true,
        },
    );

    let expected_errors = vec![TransactionError::DisputeAlreadySettled { client: 1, tx: 7 }];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn resolve_after_chargeback() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 7,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Dispute,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
        TransactionRecord {
            transaction_type: TransactionType::Chargeback,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
        TransactionRecord {
            transaction_type: TransactionType::Resolve,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 0.0,
            held: 0.0,
            locked: true,
        },
    );

    let expected_errors = vec![TransactionError::TxNotDisputed { client: 1, tx: 7 }];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn chargeback_after_resolve() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 7,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Dispute,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
        TransactionRecord {
            transaction_type: TransactionType::Resolve,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
        TransactionRecord {
            transaction_type: TransactionType::Chargeback,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 10.0,
            held: 0.0,
            locked: false,
        },
    );

    let expected_errors = vec![TransactionError::TxNotDisputed { client: 1, tx: 7 }];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn deposit_after_chargeback() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 7,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Dispute,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
        TransactionRecord {
            transaction_type: TransactionType::Chargeback,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 63,
            amount: Some(19.2),
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 0.0,
            held: 0.0,
            locked: true,
        },
    );

    let expected_errors = vec![TransactionError::AccountLocked { client: 1, tx: 63 }];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn withdrawal_after_chargeback() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 7,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Dispute,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
        TransactionRecord {
            transaction_type: TransactionType::Chargeback,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
        TransactionRecord {
            transaction_type: TransactionType::Withdrawal,
            client_id: 1,
            tx_id: 63,
            amount: Some(19.2),
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 0.0,
            held: 0.0,
            locked: true,
        },
    );

    let expected_errors = vec![TransactionError::AccountLocked { client: 1, tx: 63 }];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn withdraw_too_much() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 7,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Withdrawal,
            client_id: 1,
            tx_id: 63,
            amount: Some(19.2),
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 10.0,
            held: 0.0,
            locked: false,
        },
    );

    let expected_errors = vec![TransactionError::InsufficientFunds {
        client: 1,
        tx: 63,
        available: 10.0,
        requested: 19.2,
    }];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn negative_deposit() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 7,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 63,
            amount: Some(-19.2),
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 10.0,
            held: 0.0,
            locked: false,
        },
    );

    let expected_errors = vec![TransactionError::AmountNotPositive {
        tx: 63,
        amount: -19.2,
    }];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn negative_withdrawal() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 7,
            amount: Some(10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Withdrawal,
            client_id: 1,
            tx_id: 63,
            amount: Some(-19.2),
        },
    ];

    let mut final_accounts = HashMap::new();
    final_accounts.insert(
        1,
        Account {
            available: 10.0,
            held: 0.0,
            locked: false,
        },
    );

    let expected_errors = vec![TransactionError::AmountNotPositive {
        tx: 63,
        amount: -19.2,
    }];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}

#[test]
fn dispute_failed_tx() {
    let initial_state = State::new();

    let transactions = vec![
        TransactionRecord {
            transaction_type: TransactionType::Deposit,
            client_id: 1,
            tx_id: 7,
            amount: Some(-10.0),
        },
        TransactionRecord {
            transaction_type: TransactionType::Dispute,
            client_id: 1,
            tx_id: 7,
            amount: None,
        },
    ];

    let final_accounts = HashMap::new();

    let expected_errors = vec![
        TransactionError::AmountNotPositive {
            tx: 7,
            amount: -10.0,
        },
        TransactionError::DisputedTxFailed { tx: 7 },
    ];

    run_test_scenario(initial_state, transactions, final_accounts, expected_errors);
}
