use rand::{thread_rng, Rng};

use crate::currency::floor_currency;
use crate::handlers::handle_transaction;
use crate::state::State;
use crate::types::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};
use crate::types::{ClientId, CurrencyFloat, TransactionId};
use crate::types::{TransactionRecord, TransactionType};

const MIN_AMOUNT: CurrencyFloat = 0.0001;

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

    fn get_disputed_tx_id_for_client(&self, client_id: ClientId) -> Option<TransactionId> {
        let disputed_tx_ids = self.state.disputes.get_tx_ids_by_client(client_id);
        if disputed_tx_ids.len() > 0 {
            let all_tx_ids = self.state.transactions.get_tx_ids_by_client(client_id);
            // The set difference yields all elements of the first set but not the second
            let undisputed_tx_ids = &all_tx_ids - &disputed_tx_ids;
            undisputed_tx_ids.iter().next().cloned()
        } else {
            None
        }
    }

    fn get_undisputed_tx_id_for_client(&self, client_id: ClientId) -> Option<TransactionId> {
        let disputed_tx_ids = self.state.disputes.get_tx_ids_by_client(client_id);
        disputed_tx_ids.iter().next().cloned()
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

        if self.max_deposit > MIN_AMOUNT {
            let deposit = Deposit {
                client_id,
                tx_id: self.tx_id,
                amount: rng.gen_range(MIN_AMOUNT..self.max_deposit),
            };
            Some(deposit.into())
        } else {
            None
        }
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
                if max_amount > MIN_AMOUNT {
                    let withdrawal = Withdrawal {
                        client_id,
                        tx_id: self.tx_id,
                        amount: rng.gen_range(MIN_AMOUNT..max_amount),
                    };
                    return Some(withdrawal.into());
                }
            }
        }

        None
    }

    /// Generate a dispute for a random client if possible
    fn generate_dispute(&self) -> Option<TransactionRecord> {
        let mut rng = thread_rng();
        let client_id = self.get_client_id(&mut rng);
        if let Some(_) = self.state.accounts.get(client_id) {
            if let Some(tx_id) = self.get_undisputed_tx_id_for_client(client_id) {
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
            if let Some(tx_id) = self.get_disputed_tx_id_for_client(client_id) {
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
            if let Some(tx_id) = self.get_disputed_tx_id_for_client(client_id) {
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

            // Log progress every 10%
            let tenth = desired / 10;
            let div = self.tx_id / tenth;
            let rem = self.tx_id % tenth;
            if rem == 0 {
                log::info!("Generating transactions: {}% complete", 10 * div);
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

#[cfg(test)]
mod tests {
    use super::TransactionGenerator;
    use crate::handlers::handle_transaction;
    use crate::state::State;

    #[test]
    fn test_transaction_sequence_is_valid() {
        let num_tx = Some(10000);
        let max_client = 300;
        let max_deposit = 500.0;
        let max_attempts = 10_000;
        let generator = TransactionGenerator::new(num_tx, max_client, max_deposit, max_attempts);
        let mut state = State::new();
        for record in generator {
            let result = handle_transaction(record, &mut state);
            assert!(matches!(result, Ok(_)))
        }
    }
}
