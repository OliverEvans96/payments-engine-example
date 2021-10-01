use rand::distributions::{Distribution, Standard};
use rand::{thread_rng, Rng};

use crate::currency::floor_currency;
use crate::handlers::handle_transaction;
use crate::state::State;
use crate::types::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};
use crate::types::{ClientId, CurrencyFloat, TransactionId};
use crate::types::{TransactionRecord, TransactionType};

const MIN_AMOUNT: CurrencyFloat = 0.0001;

// Proportions of randomly generated types
// to fall in each category
// NOTE: The real, proper way to do this might
// be to implement a custom rand::Distribution,
// but I'm not going to do that.
const DEPOSIT_PCNT: f32 = 0.5;
const WITHDRAWAL_PCNT: f32 = 0.4;
const DISPUTE_PCNT: f32 = 0.05;
const RESOLVE_PCNT: f32 = 0.04;
// const CHARGEBACK_PCNT: f32 = 0.01;

const CUM_DEPOSIT: f32 = 0.0 + DEPOSIT_PCNT;
const CUM_WITHDRAWAL: f32 = CUM_DEPOSIT + WITHDRAWAL_PCNT;
const CUM_DISPUTE: f32 = CUM_WITHDRAWAL + DISPUTE_PCNT;
const CUM_RESOLVE: f32 = CUM_DISPUTE + RESOLVE_PCNT;
// const CUM_CHARGEBACK: f32 = CUM_RESOLVE + CHARGEBACK_PCNT;

impl Distribution<TransactionType> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> TransactionType {
        // Inspired by https://stackoverflow.com/a/58434531/4228052

        let x: f32 = rng.gen();

        match x {
            x if x < CUM_DEPOSIT => TransactionType::Deposit,
            x if x < CUM_WITHDRAWAL => TransactionType::Withdrawal,
            x if x < CUM_DISPUTE => TransactionType::Dispute,
            x if x < CUM_RESOLVE => TransactionType::Resolve,
            _ => TransactionType::Chargeback,
        }
    }
}

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
        let disputed_tx_ids = self.state.disputes.get_disputed_tx_ids_by_client(client_id);
        disputed_tx_ids.iter().next().cloned()
    }

    fn get_undisputed_tx_id_for_client(&self, client_id: ClientId) -> Option<TransactionId> {
        let all_tx_ids = self.state.transactions.get_tx_ids_by_client(client_id);
        let disputed_tx_ids = self.state.disputes.get_disputed_tx_ids_by_client(client_id);
        let settled_tx_ids = self.state.disputes.get_settled_tx_ids_by_client(client_id);
        // The set difference yields all elements of the first set but not the second
        let undisputed_tx_ids = &(&all_tx_ids - &disputed_tx_ids) - &settled_tx_ids;
        undisputed_tx_ids.iter().next().cloned()
    }

    /// Returns true if the (client_id, tx_id) pair is valid and of a disputable type.
    /// If any of the following are true, return false:
    /// 1. the pair is invalid
    /// 2. the transaction failed
    /// 3. or the transaction type is not disputable
    fn is_transaction_disputable(&self, client_id: ClientId, tx_id: TransactionId) -> bool {
        if let Some(tx) = self.state.transactions.get(client_id, tx_id) {
            if let Ok(Ok(_)) = tx.try_get_disputable() {
                return true;
            }
        }
        false
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
                if self.is_transaction_disputable(client_id, tx_id) {
                    let dispute = Dispute { client_id, tx_id };
                    return Some(dispute.into());
                }
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

/// Generate a random sequence of valid transactions.
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
