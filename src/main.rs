use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::error::Error;

type ClientId = u16;
type TransactionId = u32;
// Only need 4 decimals precision - f64 would be overkill
type CurrencyFloat = f32;

/// A single row in the final output CSV
struct OutputRecord {
    /// Total funds available: should equal `total` - `held`
    available: CurrencyFloat,
    /// Total disputed funds: should equal `total` - `available`
    held: CurrencyFloat,
    /// Total funds, available or otherwise: should equal `available` + `held`
    total: CurrencyFloat,
    /// Whether the account is locked: should be lock if a charge-back has occurred
    locked: bool,
}

enum TransactionError {
    InsufficientBalance {
        required: CurrencyFloat,
        actual: CurrencyFloat,
    }
}

// Transaction structs

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

// TODO: Make these all optional to avoid serde errors that would break input stream.
// Instead, we should handle parsing errors asynchronously
#[derive(Debug, Deserialize)]
struct TransactionRecord {
    #[serde(rename = "type")]
    transaction_type: TransactionType,
    #[serde(rename = "client")]
    client_id: ClientId,
    #[serde(rename = "tx")]
    tx_id: TransactionId,
    amount: Option<CurrencyFloat>,
}

#[derive(Debug)]
struct Deposit {
    client_id: ClientId,
    tx_id: TransactionId,
    amount: CurrencyFloat,
}

#[derive(Debug)]
struct Withdrawal {
    client_id: ClientId,
    tx_id: TransactionId,
    amount: CurrencyFloat,
}

#[derive(Debug)]
struct Dispute {
    client_id: ClientId,
    tx_id: TransactionId,
}

#[derive(Debug)]
struct Resolve {
    client_id: ClientId,
    tx_id: TransactionId,
}

#[derive(Debug)]
struct Chargeback {
    client_id: ClientId,
    tx_id: TransactionId,
}

#[derive(Debug)]
enum TransactionContainer {
    Deposit(Deposit),
    Withdrawal(Withdrawal),
    Dispute(Dispute),
    Resolve(Resolve),
    Chargeback(Chargeback),
}

// Internal state

#[derive(Debug)]
struct Account {
    available: CurrencyFloat,
    held: CurrencyFloat,
    locked: bool,
}

// Default state for a new account
impl Default for Account {
    fn default() -> Self {
        Self {
            available: 0.0,
            held: 0.0,
            locked: false,
        }
    }
}

// TODO: avoid locking whole state to read/write

#[derive(Debug)]
struct State {
    accounts: HashMap<ClientId, Account>,
    transactions: HashMap<TransactionId, TransactionContainer>,
}

impl State {
    fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
        }
    }
}


// Handlers

fn handle_deposit(deposit: Deposit, state: &mut State) {
    // TODO: check locked
    // TODO: add transactions to state
    state.accounts
        .entry(deposit.client_id)
        // Modify account if it's present
        .and_modify(|account| account.available += deposit.amount)
        // Otherwise, lazily create new account
        .or_insert_with(|| Account {
            available: deposit.amount,
            ..Default::default()
        });
}

fn handle_withdrawal(withdrawal: Withdrawal, state: &mut State) -> Result<(), TransactionError> {
    // TODO: check locked
    // TODO: add transactions to state
    // Get account or create one if not present
    let account = state.accounts
        .entry(withdrawal.client_id)
        .or_default();

    if account.available > withdrawal.amount {
        // Withdraw if sufficient funds are available
        account.available -= withdrawal.amount;
        Ok(())
    } else {
        // Otherwise, do nothing
        Err(TransactionError::InsufficientBalance {
            required: withdrawal.amount,
            actual: account.available
        })
    }
}

fn handle_dispute(dispute: Dispute, state: &mut State) {
    unimplemented!()
}

fn handle_resolve(resolve: Resolve, state: &mut State) {
    unimplemented!()
}

fn handle_chargeback(chargeback: Chargeback, state: &mut State) {
    unimplemented!()
}

fn handle_transaction(record: TransactionRecord, state: &mut State) {
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
            handle_deposit(deposit, state);
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
            handle_withdrawal(withdrawal, state);
        }
        TransactionRecord {
            transaction_type: TransactionType::Dispute,
            client_id,
            tx_id,
            amount: None,
        } => {
            let dispute = Dispute { client_id, tx_id };
            handle_dispute(dispute, state);
        }
        TransactionRecord {
            transaction_type: TransactionType::Resolve,
            client_id,
            tx_id,
            amount: None,
        } => {
            let resolve = Resolve { client_id, tx_id };
            handle_resolve(resolve, state);
        }
        TransactionRecord {
            transaction_type: TransactionType::Chargeback,
            client_id,
            tx_id,
            amount: None,
        } => {
            let chargeback = Chargeback { client_id, tx_id };
            handle_chargeback(chargeback, state);
        }
        _ => {
            // TODO: Handle this
            log::error!("invalid transaction")
        }
    }
}

// TODO: CL args
fn main() -> Result<(), Box<dyn Error>> {
    // Allow log level to be set via env vars without recompiling
    env_logger::init();

    // First arg is path to executable, not important
    let mut args = env::args().skip(1);

    // TODO: Clean up CLI, add help, etc.
    let input_csv_path = args
        .next()
        .expect("Missing required command line argument - input csv path");
    log::info!("reading input CSV: {}", input_csv_path);
    if args.len() > 0 {
        log::warn!(
            "unused command line arguments: {:?}",
            args.collect::<Vec<_>>()
        );
    }

    // TODO: Async / multithreaded?
    let mut state = State::new();

    if let Ok(mut reader) = csv::ReaderBuilder::new()
        // Trim whitespace before/after commas
        .trim(csv::Trim::All)
        .from_path(&input_csv_path)
    {
        let headers = reader.headers();
        println!("headers = {:?}", headers);
        for result in reader.deserialize() {
            let record: TransactionRecord = result?;
            println!("record: {:#?}", record);
            handle_transaction(record, &mut state);
        }
    } else {
        log::error!("Could not read from file {}", input_csv_path);
    }

    Ok(())
}
