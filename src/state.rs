use std::collections::{HashMap, HashSet};

use crate::account::AccountAccess;
use crate::types::{Account, TransactionContainer};
use crate::types::{ClientId, TransactionId};

// TODO: avoid locking whole state to read/write

#[derive(Debug, PartialEq)]
pub struct AccountsState(HashMap<ClientId, Account>);

impl Default for AccountsState {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl From<HashMap<ClientId, Account>> for AccountsState {
    fn from(inner: HashMap<ClientId, Account>) -> Self {
        Self(inner)
    }
}

impl AccountsState {
    pub fn get(&self, client_id: ClientId) -> Option<&Account> {
        self.0.get(&client_id)
    }

    pub fn get_or_default(&mut self, client_id: ClientId) -> &Account {
        self.0.entry(client_id).or_default()
    }

    pub fn get_mut<'a>(&'a mut self, client_id: ClientId) -> Option<AccountAccess<'a>> {
        if let Some(account) = self.0.get_mut(&client_id) {
            Some(account.access())
        } else {
            None
        }
    }

    pub fn get_mut_or_default<'a>(&'a mut self, client_id: ClientId) -> AccountAccess<'a> {
        self.0.entry(client_id).or_default().access()
    }

    /// Iterate over accounts: (client_id, account)
    pub fn iter(&self) -> impl Iterator<Item = (&ClientId, &Account)> {
        self.0.iter()
    }
}

#[derive(Debug)]
pub struct TransactionsState(HashMap<TransactionId, TransactionContainer>);

impl Default for TransactionsState {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl TransactionsState {
    pub fn get(&self, tx_id: TransactionId) -> Option<&TransactionContainer> {
        self.0.get(&tx_id)
    }

    pub fn insert(&mut self, tx_id: TransactionId, transaction: TransactionContainer) {
        // NOTE: Discarding duplicate transactions silently
        self.0.entry(tx_id).or_insert(transaction);
    }

    pub fn iter_unordered(&self) -> impl Iterator<Item=(&TransactionId, &TransactionContainer)> {
        self.0.iter()
    }
}

#[derive(Debug)]
pub struct DisputesState(HashSet<TransactionId>);

impl Default for DisputesState {
    fn default() -> Self {
        Self(HashSet::new())
    }
}

impl DisputesState {
    pub fn is_disputed(&self, tx_id: TransactionId) -> bool {
        self.0.contains(&tx_id)
    }

    pub fn dispute_tx(&mut self, tx_id: TransactionId) {
        let success = self.0.insert(tx_id);
        if !success {
            log::warn!("Transaction {} has been doubly disputed.", tx_id);
        }
    }

    pub fn undispute_tx(&mut self, tx_id: TransactionId) {
        let success = self.0.remove(&tx_id);
        if !success {
            log::warn!(
                "Transaction {} has been undisputed, but wasn't previously disputed.",
                tx_id
            );
        }
    }
}

#[derive(Debug)]
pub struct State {
    pub accounts: AccountsState,
    // TODO: log disputes, resolutions, & chargebacks?
    pub transactions: TransactionsState,
    pub disputes: DisputesState,
}

impl State {
    pub fn new() -> Self {
        Self {
            accounts: Default::default(),
            transactions: Default::default(),
            disputes: Default::default(),
        }
    }
}
