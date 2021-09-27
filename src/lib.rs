mod balances;
mod handlers;
mod output;
mod record;
pub mod test_utils;
pub mod types;
mod validate;

use std::error::Error;
use std::io;

use types::State;
use types::TransactionRecord;

pub fn process_transactions<R: io::Read, W: io::Write>(
    input_stream: &mut R,
    output_stream: &mut W,
) -> Result<(), Box<dyn Error>> {
    let mut reader = csv::ReaderBuilder::new()
        // Trim whitespace before/after commas
        .trim(csv::Trim::All)
        .from_reader(input_stream);

    // TODO: Async / multithreaded?
    let mut state = State::new();

    for result in reader.deserialize() {
        let record: TransactionRecord = result?;
        handlers::handle_transaction(record, &mut state);
    }

    output::report_balances(&state, output_stream);

    Ok(())
}
