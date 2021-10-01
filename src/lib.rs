mod account;
mod conversions;
mod currency;
mod handlers;
pub mod rand;
pub mod state;
pub mod test_utils;
mod traits;
pub mod types;
mod validate;

use csv::StringRecord;
use rayon::prelude::*;
use std::error::Error;
use std::io;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread;

use state::State;
use types::{OutputRecord, TransactionRecord};

/// Read CSV string records from a stream and send them
/// across a channel to be deserialized elsewhere.
fn read_string_records_inner<R: io::Read + Send>(
    input: R,
    headers_snd: SyncSender<StringRecord>,
    records_snd: SyncSender<Vec<StringRecord>>,
    batch_size: usize,
) -> Result<(), Box<dyn Error>> {
    // TODO: Optionally trim
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(input);

    let headers = reader.headers()?;
    headers_snd.send(headers.clone())?;

    let mut records_iter = reader.records();

    loop {
        let batch: Vec<_> = (&mut records_iter)
            .take(batch_size)
            .filter_map(Result::ok)
            .collect();
        if batch.len() > 0 {
            records_snd.send(batch)?;
        } else {
            break;
        }
    }

    Ok(())
}

/// Thin error-handling wrapper around `read_string_records_inner`
fn read_string_records<R: io::Read + Send>(
    input: R,
    headers_snd: SyncSender<StringRecord>,
    records_snd: SyncSender<Vec<StringRecord>>,
    batch_size: usize,
) {
    if let Err(err) = read_string_records_inner(input, headers_snd, records_snd, batch_size) {
        log::error!("Error while reading: {}", err);
    }
}

/// Deserialize a single CSV string record.
fn deserialize_record(record: StringRecord, headers: &StringRecord) -> Option<TransactionRecord> {
    match record.deserialize(Some(headers)) {
        Ok(ab) => Some(ab),
        Err(err) => {
            log::error!("Error while deserializing: {}", err);
            None
        }
    }
}

/// Set the number of workers in rayon's global
/// thread pool to dedicate to CSV deserialization.
pub fn configure_deserialize_workers(num_workers: Option<usize>) {
    // Default to half of the available logical cores
    let num_threads = num_workers.unwrap_or_else(|| num_cpus::get() / 2);

    let config_result = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global();

    if let Err(err) = config_result {
        log::error!("Error configuring rayon thread pool: {}", err);
    }
}

/// Read CSV records from an input stream and write them to an output stream.
/// Transactions are deserialized in parallel, but currently handled serially.
pub fn process_transactions<R: io::Read + Send + 'static, W: io::Write>(
    input_stream: R,
    output_stream: &mut W,
    batch_size: usize,
) {
    // TODO: Async / multithreaded?
    let mut state = State::new();

    // Maximum number of batches to keep in the channel at once.
    // Once this limit is reached, IO will pause until one is processed.
    let max_batches = 1;

    let (records_snd, records_rcv) = sync_channel::<Vec<StringRecord>>(max_batches);
    let (headers_snd, headers_rcv) = sync_channel::<StringRecord>(1);

    let reader_handle = thread::spawn(move || {
        read_string_records(input_stream, headers_snd, records_snd, batch_size)
    });

    if let Ok(headers) = headers_rcv.recv() {
        for batch in records_rcv {
            let tx_batch: Vec<_> = batch
                .into_par_iter()
                .filter_map(|record| deserialize_record(record, &headers))
                .collect();

            for tx in tx_batch {
                if let Err(err) = handlers::handle_transaction(tx, &mut state) {
                    log::error!("Error while handling transaction: {}", err);
                }
            }
        }
    } else {
        log::error!("Failed to get CSV headers from reader thread");
    }

    write_balances(state, output_stream);

    // Should already have finished, but wait just in case
    if let Err(err) = reader_handle.join() {
        log::error!("Failed to join reader thread: {:?}", err);
    }
}

/// Write final account balances to an output stream, consuming the state.
pub fn write_balances<W: io::Write>(state: State, output_stream: W) {
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