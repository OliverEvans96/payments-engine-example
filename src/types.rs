use serde::{Deserialize, Serialize};
use std::error::Error;
use rand::distributions::{Distribution,Standard};
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

// TODO: Test outputrecord formatting

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
    /// This is an attempt to dispute a
    /// transaction on another client's account,
    DisputeClientMismatch {
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

impl Distribution<TransactionType> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Standard {
        // Inspired by https://stackoverflow.com/a/58434531/4228052
        let x: f32 = rng.gen();
        match x {
            x if x < 0.2 => TransactionType::Deposit,
            x if x < 0.4 => TransactionType::Withdrawal,
            x if x < 0.6 => TransactionType::Dispute,
            x if x < 0.8 => TransactionType::Resolve,
            _ => TransactionType::Chargeback,
        }
    }
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

/// Allow transaction records to be randomly generated
impl Distribution<TransactionRecord> for Standard {
    fn sample_iter<R>(self, rng: R) -> rand::distributions::DistIter<Self, R, TransactionRecord>
    where
            R: rand::Rng,
            Self: Sized, {
            let max_amount = 10_000;
            let max_client = 100;

        let transaction_type: TransactionType = rng.gen();
        match transaction_type {
            TransactionType::Deposit => Deposit {
                tx_id:  0,
                client_id: rng.gen_range(0..max_client),
                amount: rng.gen_range(0..max_amount),
            },
            _ => Withdrawal {
                tx_id:  0,
                client_id: rng.gen_range(0..max_client),
                amount: rng.gen_range(0..max_amount),
            }
        };
        
    }
}

#[derive(Debug, PartialEq)]
pub struct Deposit {
    pub client_id: ClientId,
    pub tx_id: TransactionId,
    pub amount: CurrencyFloat,
}
impl Transaction for Deposit {
    #[inline]
    fn get_tx_id(&self) -> TransactionId {
        self.tx_id
    }

    #[inline]
    fn get_client_id(&self) -> ClientId {
        self.client_id
    }
}

#[derive(Debug, PartialEq)]
pub struct Withdrawal {
    pub client_id: ClientId,
    pub tx_id: TransactionId,
    pub amount: CurrencyFloat,
}
impl Transaction for Withdrawal {
    #[inline]
    fn get_tx_id(&self) -> TransactionId {
        self.tx_id
    }

    #[inline]
    fn get_client_id(&self) -> ClientId {
        self.client_id
    }
}

#[derive(Debug, PartialEq)]
pub struct Dispute {
    pub client_id: ClientId,
    pub tx_id: TransactionId,
}
impl Transaction for Dispute {
    #[inline]
    fn get_tx_id(&self) -> TransactionId {
        self.tx_id
    }

    #[inline]
    fn get_client_id(&self) -> ClientId {
        self.client_id
    }
}

#[derive(Debug, PartialEq)]
pub struct Resolve {
    pub client_id: ClientId,
    pub tx_id: TransactionId,
}
impl Transaction for Resolve {
    #[inline]
    fn get_tx_id(&self) -> TransactionId {
        self.tx_id
    }

    #[inline]
    fn get_client_id(&self) -> ClientId {
        self.client_id
    }
}

#[derive(Debug, PartialEq)]
pub struct Chargeback {
    pub client_id: ClientId,
    pub tx_id: TransactionId,
}

impl Transaction for Chargeback {
    #[inline]
    fn get_tx_id(&self) -> TransactionId {
        self.tx_id
    }

    #[inline]
    fn get_client_id(&self) -> ClientId {
        self.client_id
    }
}

pub trait Transaction {
    fn get_tx_id(&self) -> TransactionId;
    fn get_client_id(&self) -> ClientId;
}

/// This trait indicates whether and how a transaction can be disputed.
/// To enable new types of transactions to be disputed, implement this
/// trait for that type, and update TransactionContainer::try_get_disputable.
pub trait Disputable: Transaction {
    fn modify_balances_for_dispute(&self, account: &mut Account);
    fn modify_balances_for_resolve(&self, account: &mut Account);
    fn modify_balances_for_chargeback(&self, account: &mut Account);
}

impl Disputable for Deposit {
    fn modify_balances_for_dispute(&self, account: &mut Account) {
        account.available -= self.amount;
        account.held += self.amount;
    }
    fn modify_balances_for_resolve(&self, account: &mut Account) {
        account.available += self.amount;
        account.held -= self.amount;
    }
    fn modify_balances_for_chargeback(&self, account: &mut Account) {
        account.held -= self.amount;
    }
}

/// This transaction must follow a dispute with the same tx_id and client_id
pub trait PostDispute: Transaction {}

impl PostDispute for Resolve {}
impl PostDispute for Chargeback {}

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

    pub fn try_get_disputable(
        &self,
    ) -> Result<&Result<impl Disputable, TransactionError>, TransactionType> {
        match self {
            // NOTE: Only deposits may be disputed
            TransactionContainer::Deposit(result) => Ok(result),
            other => Err(other.tx_type()),
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
