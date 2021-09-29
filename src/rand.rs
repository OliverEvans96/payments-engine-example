use rand::{thread_rng, Rng};

use crate::currency::floor_currency;
use crate::handlers::handle_transaction;
use crate::state::State;
use crate::types::{Chargeback, Deposit, Dispute, Resolve, TransactionContainer, Withdrawal};
use crate::types::{ClientId, CurrencyFloat, TransactionId};
use crate::types::{TransactionRecord, TransactionType};

const MIN_AMOUNT: CurrencyFloat = 0.001;

struct TransactionGenerator {
    state: State,
    tx_id: TransactionId,
    num_tx: Option<TransactionId>,
    max_client: ClientId,
    max_deposit: CurrencyFloat,
    max_attempts: usize,
}

impl TransactionGenerator {
    fn new(
        num_tx: Option<TransactionId>,
        max_client: ClientId,
        max_deposit: CurrencyFloat,
        max_attempts: usize,
    ) -> Self {
        Self {
            state: State::new(),
            tx_id: 1,
            num_tx,
            max_client,
            max_deposit,
            max_attempts,
        }
    }
}

impl TransactionGenerator {
    fn get_client_id<R: Rng>(&self, rng: &mut R) -> ClientId {
        rng.gen_range(1..=self.max_client)
    }

    /// NOTE: This is a very expensive way to do this.
    /// It would be much easier if transactions were stored
    /// grouped by client.
    fn get_txs_for_client(
        &self,
        client_id: ClientId,
    ) -> Vec<(TransactionId, &TransactionContainer)> {
        let mut txs = Vec::new();
        // Iterate over ALL previous transactions
        for (&tx_id, tx) in self.state.transactions.iter_unordered() {
            // Downcast successful transactions to Box<dyn Transaction>
            if let Ok(boxed) = tx.get_transaction() {
                // If transaction is for the relevant client
                if boxed.get_client_id() == client_id {
                    txs.push((tx_id, tx));
                }
            }
        }
        txs
    }

    /// Get a single transaction for a client which is either disputed or undisputed,
    /// depending on the `disputed` arg, if one exists.
    /// TODO: This is so terribly inefficient
    fn get_disputable_tx_id_for_client(
        &self,
        client_id: ClientId,
        disputed: bool,
    ) -> Option<TransactionId> {
        let client_txs_ids = self.get_txs_for_client(client_id);
        for (tx_id, tx) in client_txs_ids {
            if let Ok(_) = tx.try_get_disputable() {
                if self.state.disputes.is_disputed(tx_id) == disputed {
                    return Some(tx_id);
                }
            }
        }
        None
    }

    /// Generate a deposit for a random client if possible
    fn generate_deposit(&self) -> Option<TransactionRecord> {
        let mut rng = thread_rng();
        let client_id = self.get_client_id(&mut rng);
        if let Some(account) = self.state.accounts.get(client_id) {
            if account.locked {
                return None;
            }
        }

        let deposit = Deposit {
            client_id,
            tx_id: self.tx_id,
            amount: rng.gen_range(MIN_AMOUNT..self.max_deposit),
        };

        Some(deposit.into())
    }

    /// Generate a withdrawal for a random client if possible
    fn generate_withdrawal(&self) -> Option<TransactionRecord> {
        let mut rng = thread_rng();
        let client_id = self.get_client_id(&mut rng);
        if let Some(account) = self.state.accounts.get(client_id) {
            if !account.locked && account.available > MIN_AMOUNT {
                // Floor here to make sure amount doesn't exceed
                // the available balance after rounding.
                let max_amount = floor_currency(account.available);
                let withdrawal = Withdrawal {
                    client_id,
                    tx_id: self.tx_id,
                    amount: rng.gen_range(MIN_AMOUNT..max_amount),
                };
                return Some(withdrawal.into());
            }
        }

        None
    }

    /// Generate a dispute for a random client if possible
    fn generate_dispute(&self) -> Option<TransactionRecord> {
        let mut rng = thread_rng();
        let client_id = self.get_client_id(&mut rng);
        if let Some(_) = self.state.accounts.get(client_id) {
            if let Some(tx_id) = self.get_disputable_tx_id_for_client(client_id, false) {
                let dispute = Dispute { client_id, tx_id };
                return Some(dispute.into());
            }
        }
        None
    }

    /// Generate a resolve for a random client if possible
    fn generate_resolve(&self) -> Option<TransactionRecord> {
        let mut rng = thread_rng();
        let client_id = self.get_client_id(&mut rng);
        if let Some(_) = self.state.accounts.get(client_id) {
            if let Some(tx_id) = self.get_disputable_tx_id_for_client(client_id, true) {
                let resolve = Resolve { client_id, tx_id };
                return Some(resolve.into());
            }
        }
        None
    }

    fn generate_chargeback(&self) -> Option<TransactionRecord> {
        let mut rng = thread_rng();
        let client_id = self.get_client_id(&mut rng);
        if let Some(_) = self.state.accounts.get(client_id) {
            if let Some(tx_id) = self.get_disputable_tx_id_for_client(client_id, true) {
                let chargeback = Chargeback { client_id, tx_id };
                return Some(chargeback.into());
            }
        }
        None
    }

    fn generate_potential_transaction(&mut self) -> Option<TransactionRecord> {
        let mut rng = thread_rng();
        let transaction_type: TransactionType = rng.gen();
        match transaction_type {
            // TODO: Move these to functions
            TransactionType::Deposit => self.generate_deposit(),
            TransactionType::Withdrawal => self.generate_withdrawal(),
            TransactionType::Dispute => self.generate_dispute(),
            TransactionType::Resolve => self.generate_resolve(),
            TransactionType::Chargeback => self.generate_chargeback(),
        }
    }
}

impl Iterator for TransactionGenerator {
    type Item = TransactionRecord;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(desired) = self.num_tx {
            // Maybe break early
            if self.tx_id > desired {
                return None;
            }
        }

        // NOTE: it's possible that all accounts are locked, all disputes are resolve,
        // and no further transactions can be generated.
        for _ in 0..self.max_attempts {
            if let Some(tx) = self.generate_potential_transaction() {
                handle_transaction(tx.clone(), &mut self.state)
                    .expect("Generated invalid transaction");
                self.tx_id += 1;
                return Some(tx);
            }
        }

        log::error!("Reached max attempts to generate new transaction.");

        None
    }
}

pub fn generate_random_valid_transaction_sequence(
    num_tx: Option<TransactionId>,
    max_client: ClientId,
    max_deposit: CurrencyFloat,
    max_attempts: usize,
) -> impl Iterator<Item = TransactionRecord> {
    let generator = TransactionGenerator::new(num_tx, max_client, max_deposit, max_attempts);
    generator.into_iter()
}
