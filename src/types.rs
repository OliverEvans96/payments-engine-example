use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Debug, Display};

use crate::currency::round_currency;
pub use crate::currency::CurrencyFloat;

pub type ClientId = u16;
pub type TransactionId = u32;

/// A single row in the final output CSV
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct OutputRecord {
    /// Id for client's account
    pub client: ClientId,
    /// Total funds available: should equal `total` - `held`
    pub available: CurrencyFloat,
    /// Total disputed funds: should equal `total` - `available`
    pub held: CurrencyFloat,
    /// Total funds, available or otherwise: should equal `available` + `held`
    pub total: CurrencyFloat,
    /// Whether the account is locked: should be lock if a charge-back has occurred
    pub locked: bool,
}

impl OutputRecord {
    pub fn new(client_id: ClientId, account: &Account) -> Self {
        OutputRecord {
            client: client_id,
            // NOTE: Rounding just in case some strange floating point phemonenon added extra digits
            // It's still possible that this would still format to more than four digits,
            // but it's a lot easier than writing a custom serializer / deserializer
            available: round_currency(account.available),
            held: round_currency(account.held),
            total: round_currency(account.available + account.held),
            locked: account.locked,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TransactionError {
    /// Client attempted to withdraw more than their available funds.
    InsufficientFunds {
        client: ClientId,
        tx: TransactionId,
        requested: CurrencyFloat,
        available: CurrencyFloat,
    },
    /// This account is locked, and cannot deposit or withdraw.
    AccountLocked { client: ClientId, tx: TransactionId },
    /// Transaction IDs must be globally unique.
    DuplicateTxId { tx: TransactionId },
    /// Deposits and withdrawals must have positive amounts.
    AmountNotPositive {
        tx: TransactionId,
        amount: CurrencyFloat,
    },
    /// Cannot dispute an actively disputed transaction.
    TxAlreadyDisputed { client: ClientId, tx: TransactionId },
    /// Dispute refers to nonexistent transaction.
    TxDoesNotExist { client: ClientId, tx: TransactionId },
    /// Only deposits can be disputed.
    InvalidDispute {
        tx: TransactionId,
        tx_type: TransactionType,
    },
    /// An undisputed transaction cannot
    /// be resolved or charged back,
    TxNotDisputed { client: ClientId, tx: TransactionId },
    /// The disputed transaction didn't succeed,
    /// so there's no point in disputing it.
    DisputedTxFailed { tx: TransactionId },
    /// Transaction has already been disputed and settled - cannot redispute.
    DisputeAlreadySettled { client: ClientId, tx: TransactionId },
    /// The client_id on this transaction does not
    /// match the client_id on the referenced transaction.
    ClientMismatch {
        tx: TransactionId,
        tx_client: ClientId,
        dispute_client: ClientId,
    },
    /// Transaction had unknown type or missing required fields.
    ImproperTransaction(TransactionRecord),
    /// Didn't think we'd ever get here, but here we are.
    UnexpectedError(String),
}

impl Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl Error for TransactionError {}

// Transaction structs

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TransactionRecord {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    #[serde(rename = "client")]
    pub client_id: ClientId,
    #[serde(rename = "tx")]
    pub tx_id: TransactionId,
    pub amount: Option<CurrencyFloat>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Deposit {
    pub client_id: ClientId,
    pub tx_id: TransactionId,
    pub amount: CurrencyFloat,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Withdrawal {
    pub client_id: ClientId,
    pub tx_id: TransactionId,
    pub amount: CurrencyFloat,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dispute {
    pub client_id: ClientId,
    pub tx_id: TransactionId,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Resolve {
    pub client_id: ClientId,
    pub tx_id: TransactionId,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Chargeback {
    pub client_id: ClientId,
    pub tx_id: TransactionId,
}

#[derive(Debug, PartialEq)]
pub enum TransactionContainer {
    Deposit(Result<Deposit, TransactionError>),
    Withdrawal(Result<Withdrawal, TransactionError>),
}

impl TransactionContainer {
    pub fn tx_type(&self) -> TransactionType {
        match &self {
            TransactionContainer::Deposit(_) => TransactionType::Deposit,
            TransactionContainer::Withdrawal(_) => TransactionType::Withdrawal,
        }
    }
}


// Internal state

#[derive(Debug, PartialEq)]
pub struct Account {
    pub available: CurrencyFloat,
    pub held: CurrencyFloat,
    pub locked: bool,
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
