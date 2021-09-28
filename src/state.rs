use std::collections::{HashMap, HashSet};

use crate::account::AccountAccess;
use crate::types::{Account, TransactionContainer};
use crate::types::{ClientId, TransactionId};

// TODO: avoid locking whole state to read/write

#[derive(Debug, PartialEq)]
pub struct State {
    accounts: HashMap<ClientId, Account>,
    // TODO: log disputes, resolutions, & chargebacks?
    pub transactions: HashMap<TransactionId, TransactionContainer>,
    pub active_disputes: HashSet<TransactionId>,
}

impl State {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
            active_disputes: HashSet::new(),
        }
    }

    pub fn get_account<'a>(&'a self, client_id: ClientId) -> Option<&'a Account> {
        self.accounts.get(&client_id)
    }

    pub fn get_account_or_default<'a>(&'a self, client_id: ClientId) -> &'a Account {
        self.accounts.entry(client_id).or_default()
    }

    pub fn iter_accounts(&self) -> std::collections::hash_map::Iter<ClientId, Account> {
        self.accounts.iter()
    }

    pub fn get_mut_account<'a>(&'a mut self, client_id: ClientId) -> Option<AccountAccess<'a>> {
        if let Some(account) = self.accounts.get(&client_id) {
            Some(account.get_container())
        } else {
            None
        }
    }

    pub fn get_mut_account_or_default<'a>(
        &'a mut self,
        client_id: ClientId,
    ) -> AccountAccess<'a> {
        let account = self.accounts.entry(client_id).or_default();
        account.get_container()
    }
}
