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

#[cfg(test)]
mod tests {
    use crate::types::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};
    use crate::types::{TransactionRecord, TransactionType};

    #[test]
    fn test_deposit_to_record() {
        let deposit = Deposit {
            amount: 3.6,
            client_id: 17,
            tx_id: 199,
        };

        let record = TransactionRecord {
            transaction_type: TransactionType::Deposit,
            amount: Some(3.6),
            client_id: 17,
            tx_id: 199,
        };

        assert_eq!(record, deposit.into());
    }

    #[test]
    fn test_withdrawal_to_record() {
        let withdrawal = Withdrawal {
            amount: 3.6,
            client_id: 17,
            tx_id: 199,
        };

        let record = TransactionRecord {
            transaction_type: TransactionType::Withdrawal,
            amount: Some(3.6),
            client_id: 17,
            tx_id: 199,
        };

        assert_eq!(record, withdrawal.into());
    }

    #[test]
    fn test_dispute_to_record() {
        let dispute = Dispute {
            client_id: 17,
            tx_id: 199,
        };

        let record = TransactionRecord {
            transaction_type: TransactionType::Dispute,
            amount: None,
            client_id: 17,
            tx_id: 199,
        };

        assert_eq!(record, dispute.into());
    }

    #[test]
    fn test_resolve_to_record() {
        let resolve = Resolve {
            client_id: 17,
            tx_id: 199,
        };

        let record = TransactionRecord {
            transaction_type: TransactionType::Resolve,
            amount: None,
            client_id: 17,
            tx_id: 199,
        };

        assert_eq!(record, resolve.into());
    }

    #[test]
    fn test_chargeback_to_record() {
        let chargeback = Chargeback {
            client_id: 17,
            tx_id: 199,
        };

        let record = TransactionRecord {
            transaction_type: TransactionType::Chargeback,
            amount: None,
            client_id: 17,
            tx_id: 199,
        };

        assert_eq!(record, chargeback.into());
    }
}
