mod account;
mod currency;
mod handlers;
mod record;
pub mod test_utils;
pub mod types;
mod validate;

use std::io;

use types::{OutputRecord, State, TransactionRecord};

// TODO: Test locked account
// TODO: Test duplicate transaction id for valid first transaction
// TODO: Test duplicate transaction id for invalid first transaction
// TODO: Test unordered transaction ids
// TODO: Test dispute referencing nonexistent transaction
// TODO: Test resolve / chargeback referencing nonexistent transaction
// TODO: Test resolve / chargeback referencing undisputed transaction
// TODO: Test dispute / resolve / chargeback with client_id not matching referenced transaction

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
        let record = OutputRecord::new(client_id, account);

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
