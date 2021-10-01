use crate::types::{Account, TransactionContainer, TransactionError, TransactionType};
use crate::types::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};
use crate::types::{ClientId, TransactionId};

pub trait Transaction {
    fn get_tx_id(&self) -> TransactionId;
    fn get_client_id(&self) -> ClientId;
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

impl TransactionContainer {
    /// Try to downcast the `TransactionContainer` to `impl Disputable`
    /// NOTE: If more than Deposit is disputable,
    /// this will have to change from `impl Disputable` to `Box<dyn Disputable>`.
    pub fn try_get_disputable(
        &self,
    ) -> Result<&Result<impl Disputable, TransactionError>, TransactionType> {
        match self {
            // NOTE: Only deposits may be disputed
            TransactionContainer::Deposit(result) => Ok(result),
            other => Err(other.tx_type()),
        }
    }

    /// Downcast the TransactionContainer to `Box<dyn Transacion>`
    pub fn get_transaction(&self) -> Result<Box<dyn Transaction>, TransactionError> {
        match self {
            TransactionContainer::Deposit(result) => {
                result.clone().map(|t| Box::new(t) as Box<dyn Transaction>)
            }
            TransactionContainer::Withdrawal(result) => {
                result.clone().map(|t| Box::new(t) as Box<dyn Transaction>)
            }
        }
    }
}
