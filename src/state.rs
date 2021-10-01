use std::collections::{HashMap, HashSet};

use crate::account::AccountAccess;
use crate::types::{Account, TransactionContainer, TransactionError};
use crate::types::{ClientId, TransactionId};

/// Component of application state dealing with accounts: balances and status.
#[derive(Debug, Default, PartialEq)]
pub struct AccountsState(HashMap<ClientId, Account>);

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

/// Record of all transactions relevant to engine operation.
/// This is not intended for logging purposes.
/// Disputes, resolves, and chargebacks are not stored since
/// they are never directly referenced by other transactions.
/// Therefore, this struct contains only deposits and withdrawals.
///
/// Both successful and failed transactions are stored
/// within TransactionContainer, which wraps a Result.
#[derive(Debug, Default)]
pub struct TransactionsState {
    by_client: HashMap<ClientId, HashMap<TransactionId, TransactionContainer>>,
    tx_ids: HashSet<TransactionId>,
}

impl TransactionsState {
    pub fn tx_exists(&self, tx_id: TransactionId) -> bool {
        self.tx_ids.contains(&tx_id)
    }

    pub fn get(&self, client_id: ClientId, tx_id: TransactionId) -> Option<&TransactionContainer> {
        self.by_client.get(&client_id).and_then(|c| c.get(&tx_id))
    }

    pub fn insert(
        &mut self,
        client_id: ClientId,
        tx_id: TransactionId,
        transaction: TransactionContainer,
    ) {
        // Get hash map for client, or create one if none exists.
        let client_txs = self.by_client.entry(client_id).or_default();

        // Store transaction id globally to avoid duplicates
        let success = self.tx_ids.insert(tx_id);
        if !success {
            log::warn!(
                "Storing duplicate tx_id {} - did you forget to validate?",
                tx_id
            )
        }

        // NOTE: Discarding duplicate transactions silently
        client_txs.entry(tx_id).or_insert(transaction);
    }

    /// Get the set of tx ids for this client
    pub fn get_tx_ids_by_client(&self, client_id: ClientId) -> HashSet<TransactionId> {
        // See https://stackoverflow.com/a/59156843/4228052
        if let Some(map) = self.by_client.get(&client_id) {
            map.keys().cloned().collect()
        } else {
            HashSet::new()
        }
    }
}

/// Current state of all disputes, past and present.
/// Once a dispute is filed for a transaction, it is
/// considered actively disputed, and its tx_id is stored
/// in the `active` field.
///
/// Once a resolve or chargeback has been filed, it is
/// considered settled, and can no longer be re-disputed.
/// These tx_ids are found in the `settled` field.
#[derive(Debug, Default)]
pub struct DisputesState {
    active: HashMap<ClientId, HashSet<TransactionId>>,
    settled: HashMap<ClientId, HashSet<TransactionId>>,
}

impl DisputesState {
    /// Determine whether a client's transaction is actively disputed.
    pub fn is_disputed(&self, client_id: ClientId, tx_id: TransactionId) -> bool {
        if let Some(client_active) = self.active.get(&client_id) {
            client_active.contains(&tx_id)
        } else {
            false
        }
    }

    /// Determine whether a client's transaction has been disputed and settled.
    pub fn is_settled(&self, client_id: ClientId, tx_id: TransactionId) -> bool {
        if let Some(client_settled) = self.settled.get(&client_id) {
            client_settled.contains(&tx_id)
        } else {
            false
        }
    }

    /// Mark a transaction as actively disputed.
    pub fn dispute_tx(
        &mut self,
        client_id: ClientId,
        tx_id: TransactionId,
    ) -> Result<(), TransactionError> {
        // TODO: These things should already be checked.
        // Can we safely avoid checking twice?
        // NOTE: Not checking whether transaction is already settled
        let client_disputes = self.active.entry(client_id).or_default();
        let insert_success = client_disputes.insert(tx_id);
        if insert_success {
            Ok(())
        } else {
            Err(TransactionError::TxAlreadyDisputed {
                client: client_id,
                tx: tx_id,
            })
        }
    }

    /// Mark a transaction as settled.
    pub fn settle_dispute(
        &mut self,
        client_id: ClientId,
        tx_id: TransactionId,
    ) -> Result<(), TransactionError> {
        // NOTE: When using async, make sure to { remove & insert } atomically.
        if let Some(client_active) = self.active.get_mut(&client_id) {
            let remove_success = client_active.remove(&tx_id);
            if remove_success {
                let client_settled = self.settled.entry(client_id).or_default();
                let insert_success = client_settled.insert(tx_id);
                if insert_success {
                    return Ok(());
                } else {
                    return Err(TransactionError::DisputeAlreadySettled {
                        tx: tx_id,
                        client: client_id,
                    });
                }
            }
        }
        Err(TransactionError::TxNotDisputed {
            client: client_id,
            tx: tx_id,
        })
    }

    /// Get the set of all disputed transaction ids for a client.
    pub fn get_disputed_tx_ids_by_client(&self, client_id: ClientId) -> HashSet<TransactionId> {
        self.active
            .get(&client_id)
            .cloned()
            .unwrap_or_else(HashSet::new)
    }

    /// Get the set of all settled transaction ids for a client.
    pub fn get_settled_tx_ids_by_client(&self, client_id: ClientId) -> HashSet<TransactionId> {
        self.settled
            .get(&client_id)
            .cloned()
            .unwrap_or_else(HashSet::new)
    }
}

/// Root application state
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
