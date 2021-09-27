mod balances;
mod handlers;
mod record;
pub mod test_utils;
pub mod types;
mod validate;

use std::io;

use types::{OutputRecord, State, TransactionRecord};

pub fn process_transactions<R: io::Read, W: io::Write>(
    input_stream: &mut R,
    output_stream: &mut W,
) {
    let mut reader = csv::ReaderBuilder::new()
        // Trim whitespace before/after commas
        .trim(csv::Trim::All)
        .from_reader(input_stream);

    // TODO: Async / multithreaded?
    let mut state = State::new();

    for result in reader.deserialize::<TransactionRecord>() {
        match result {
            Ok(record) => {
                if let Err(err) = handlers::handle_transaction(record, &mut state) {
                    log::error!("Error while handling transaction: {}", err);
                }
            }
            Err(err) => {
                log::error!("Error while parsing transaction: {}", err);
            }
        }
    }

    write_balances(&state, output_stream);
}

pub fn write_balances<W: io::Write>(state: &State, output_stream: W) {
    let mut writer = csv::Writer::from_writer(output_stream);
    for (&client_id, account) in state.accounts.iter() {
        let record = OutputRecord {
            client: client_id,
            available: account.available,
            held: account.held,
            total: account.available + account.held,
            locked: account.locked,
        };

        if let Err(err) = writer.serialize(&record) {
            log::error!("error writing serialized account balances: {}", err);
        }
    }
    if let Err(err) = writer.flush() {
        log::error!("error flusing serialized account balances: {}", err);
    }
}

#[cfg(test)]
mod tests {
    // TODO: unit tests
}
