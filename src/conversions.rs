use crate::types::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};
use crate::types::{TransactionRecord, TransactionType};

// Convert from individual transaction types
// to TransactionRecord for the sake of
// generating random valid transaction

impl From<Deposit> for TransactionRecord {
    fn from(t: Deposit) -> Self {
        Self {
            transaction_type: TransactionType::Deposit,
            client_id: t.client_id,
            tx_id: t.tx_id,
            amount: Some(t.amount),
        }
    }
}

impl From<Withdrawal> for TransactionRecord {
    fn from(t: Withdrawal) -> Self {
        Self {
            transaction_type: TransactionType::Withdrawal,
            client_id: t.client_id,
            tx_id: t.tx_id,
            amount: Some(t.amount),
        }
    }
}

impl From<Dispute> for TransactionRecord {
    fn from(t: Dispute) -> Self {
        Self {
            transaction_type: TransactionType::Dispute,
            client_id: t.client_id,
            tx_id: t.tx_id,
            amount: None,
        }
    }
}

impl From<Resolve> for TransactionRecord {
    fn from(t: Resolve) -> Self {
        Self {
            transaction_type: TransactionType::Resolve,
            client_id: t.client_id,
            tx_id: t.tx_id,
            amount: None,
        }
    }
}

impl From<Chargeback> for TransactionRecord {
    fn from(t: Chargeback) -> Self {
        Self {
            transaction_type: TransactionType::Chargeback,
            client_id: t.client_id,
            tx_id: t.tx_id,
            amount: None,
        }
    }
}
