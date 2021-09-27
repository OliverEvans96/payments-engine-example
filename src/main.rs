use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;
use std::io;

type ClientId = u16;
type TransactionId = u32;
// Only need 4 decimals precision - f64 would be overkill
type CurrencyFloat = f32;

/// A single row in the final output CSV
#[derive(Debug, Serialize)]
struct OutputRecord {
    /// Id for client's account
    client: ClientId,
    /// Total funds available: should equal `total` - `held`
    available: CurrencyFloat,
    /// Total disputed funds: should equal `total` - `available`
    held: CurrencyFloat,
    /// Total funds, available or otherwise: should equal `available` + `held`
    total: CurrencyFloat,
    /// Whether the account is locked: should be lock if a charge-back has occurred
    locked: bool,
}

#[derive(Debug)]
enum TransactionError {
    InsufficientFunds {
        required: CurrencyFloat,
        actual: CurrencyFloat,
    },
    AccountLocked,
    DuplicateTxId,
    TxAlreadyDisputed,
    TxDoesNotExist,
    InvalidDispute,
    TxNotDisputed,
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
    Deposit(Result<Deposit, TransactionError>),
    Withdrawal(Result<Withdrawal, TransactionError>),
    // Dispute(Result<Dispute, TransactionError>),
    // Resolve(Result<Resolve, TransactionError>),
    // Chargeback(Result<Chargeback, TransactionError>),
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
    // TODO: log disputes, resolutions, & chargebacks?
    transactions: HashMap<TransactionId, TransactionContainer>,
    active_disputes: HashSet<TransactionId>,
}

impl State {
    fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
            active_disputes: HashSet::new(),
        }
    }
}

// Handlers

fn check_for_duplicate_tx_id(tx_id: TransactionId, state: &State) -> Result<(), TransactionError> {
    // NOTE: discarding duplicate transactions
    // TODO: Efficiently record duplicate transactions?

    if let Some(tx) = state.transactions.get(&tx_id) {
        // Duplicate transactions are a bad sign
        Err(TransactionError::DuplicateTxId)
    } else {
        Ok(())
    }
}

fn validate_deposit(deposit: &Deposit, state: &State) -> Result<(), TransactionError> {
    check_for_duplicate_tx_id(deposit.tx_id, state)?;

    if let Some(account) = state.accounts.get(&deposit.client_id) {
        if account.locked {
            // Locked accounts cannot deposit
            return Err(TransactionError::AccountLocked);
        }
    }

    // New and unlocked accounts can deposit
    Ok(())
}

fn validate_withdrawal(withdrawal: &Withdrawal, state: &State) -> Result<(), TransactionError> {
    check_for_duplicate_tx_id(withdrawal.tx_id, state)?;

    if let Some(account) = state.accounts.get(&withdrawal.client_id) {
        if account.locked {
            // Locked accounts cannot withdraw
            return Err(TransactionError::AccountLocked);
        } else {
            // unlocked accounts can withdraw if they have enough funds
            if account.available >= withdrawal.amount {
                Ok(())
            } else {
                return Err(TransactionError::InsufficientFunds {
                    required: withdrawal.amount,
                    actual: account.available,
                });
            }
        }
    } else {
        // New accounts cannot withdraw
        // TODO: This would be a weird error for a 0-amount withdrawal
        return Err(TransactionError::InsufficientFunds {
            required: withdrawal.amount,
            actual: 0.0,
        });
    }
}

fn validate_dispute(dispute: &Dispute, state: &State) -> Result<(), TransactionError> {
    // NOTE: disputes do not have their own transaction id, they refer to a deposit or withdrawal
    // NOTE: locked accounts are still allowed to dispute, just not deposit or withdraw

    // NOTE: Cannot dispute an actively disputed transaction
    if state.active_disputes.contains(&dispute.tx_id) {
        return Err(TransactionError::TxAlreadyDisputed);
    }

    // Get disputed transaction from log
    if let Some(disputed_transaction) = state.transactions.get(&dispute.tx_id) {
        // NOTE: Only deposits may be disputed
        if let TransactionContainer::Deposit(_) = disputed_transaction {
            // TODO: Verify that disputed deposit actually succeeded
            Ok(())
        } else {
            Err(TransactionError::InvalidDispute)
        }
    } else {
        Err(TransactionError::TxDoesNotExist)
    }
}

fn validate_resolve(resolve: &Resolve, state: &State) -> Result<(), TransactionError> {
    // NOTE: resolves do not have their own transaction id, they refer to a deposit or withdrawal
    // NOTE: locked accounts are still allowed to resolve, just not deposit or withdraw

    // NOTE: Cannot resolve an undisputed transaction
    if state.active_disputes.contains(&resolve.tx_id) {
        Ok(())
    } else {
        Err(TransactionError::TxNotDisputed)
    }
}

fn validate_chargeback(chargeback: &Chargeback, state: &State) -> Result<(), TransactionError> {
    // NOTE: chargebacks do not have their own transaction id, they refer to a deposit or withdrawal
    // NOTE: locked accounts are still allowed to chargeback, just not deposit or withdraw

    // NOTE: Cannot chargeback an undisputed transaction
    if state.active_disputes.contains(&chargeback.tx_id) {
        Ok(())
    } else {
        Err(TransactionError::TxNotDisputed)
    }
}

// Balance modification

fn modify_balances_for_deposit(deposit: &Deposit, account: &mut Account) {
    account.available += deposit.amount;
}

fn modify_balances_for_withdrawal(withdrawal: &Withdrawal, account: &mut Account) {
    account.available -= withdrawal.amount;
}

fn modify_balances_for_dispute(disputed_deposit: &Deposit, account: &mut Account) {
    account.available -= disputed_deposit.amount;
    account.held += disputed_deposit.amount;
}

fn modify_balances_for_resolve(disputed_deposit: &Deposit, account: &mut Account) {
    account.available += disputed_deposit.amount;
    account.held -= disputed_deposit.amount;
}

fn modify_balances_for_chargeback(disputed_deposit: &Deposit, account: &mut Account) {
    account.held -= disputed_deposit.amount;
}

// Record transactions

// NOTE: Assuming transaction has already been validated
fn record_deposit(deposit: Deposit, state: &mut State) {
    // Update account
    state
        .accounts
        .entry(deposit.client_id)
        // Modify account if it's present
        .and_modify(|account| modify_balances_for_deposit(&deposit, account))
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
fn record_withdrawal(withdrawal: Withdrawal, state: &mut State) {
    // Since withdrawing from an account with no existing balance is invalid,
    // we can assume that account already exists (and unwrap the option)
    let account = state.accounts.get_mut(&withdrawal.client_id).unwrap();

    modify_balances_for_withdrawal(&withdrawal, account);

    // Log transaction
    state
        .transactions
        .entry(withdrawal.tx_id)
        .or_insert(TransactionContainer::Withdrawal(Ok(withdrawal)));
}

// NOTE: Assuming dispute has already been validated
fn record_dispute(dispute: Dispute, state: &mut State) {
    if let Some(TransactionContainer::Deposit(Ok(disputed_deposit))) =
        state.transactions.get(&dispute.tx_id)
    {
        // Get associated account
        if let Some(account) = state.accounts.get_mut(&dispute.client_id) {
            modify_balances_for_dispute(disputed_deposit, account);

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

fn record_resolve(resolve: Resolve, state: &mut State) {
    if let Some(TransactionContainer::Deposit(Ok(disputed_deposit))) =
        state.transactions.get(&resolve.tx_id)
    {
        // Get associated account
        if let Some(account) = state.accounts.get_mut(&resolve.client_id) {
            modify_balances_for_resolve(disputed_deposit, account);

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

fn record_chargeback(chargeback: Chargeback, state: &mut State) {
    if let Some(TransactionContainer::Deposit(Ok(disputed_deposit))) =
        state.transactions.get(&chargeback.tx_id)
    {
        // Get associated account
        if let Some(account) = state.accounts.get_mut(&chargeback.client_id) {
            modify_balances_for_chargeback(disputed_deposit, account);

            // Mark the transaction as no longer disputed
            let success = state.active_disputes.remove(&chargeback.tx_id);

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

fn handle_deposit(deposit: Deposit, state: &mut State) -> Result<(), TransactionError> {
    validate_deposit(&deposit, state)?;
    record_deposit(deposit, state);
    Ok(())
}

fn handle_withdrawal(withdrawal: Withdrawal, state: &mut State) -> Result<(), TransactionError> {
    validate_withdrawal(&withdrawal, state)?;
    record_withdrawal(withdrawal, state);
    Ok(())
}

fn handle_dispute(dispute: Dispute, state: &mut State) -> Result<(), TransactionError> {
    validate_dispute(&dispute, state)?;
    record_dispute(dispute, state);
    Ok(())
}

fn handle_resolve(resolve: Resolve, state: &mut State) -> Result<(), TransactionError> {
    validate_resolve(&resolve, state)?;
    record_resolve(resolve, state);
    Ok(())
}

fn handle_chargeback(chargeback: Chargeback, state: &mut State) -> Result<(), TransactionError> {
    validate_chargeback(&chargeback, state)?;
    record_chargeback(chargeback, state);
    Ok(())
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
            // TODO: Handle errors
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

fn report_balances(state: &State) {
    let mut writer = csv::Writer::from_writer(io::stdout());
    for (&client_id, account) in state.accounts.iter() {
        let record = OutputRecord {
            client: client_id,
            available: account.available,
            held: account.held,
            total: account.available + account.held,
            locked: account.locked,
        };

        if let Err(err) = writer.serialize(record) {
            log::error!("error writing serialized account balances: {}", err);
        }
    }
    if let Err(err) = writer.flush() {
        log::error!("error flusing serialized account balances: {}", err);
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
        for result in reader.deserialize() {
            let record: TransactionRecord = result?;
            handle_transaction(record, &mut state);
        }

        report_balances(&state);
    } else {
        log::error!("Could not read from input file '{}'", input_csv_path);
    }

    Ok(())
}
