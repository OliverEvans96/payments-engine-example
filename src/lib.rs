mod account;
mod currency;
mod handlers;
pub mod rand;
pub mod state;
pub mod test_utils;
pub mod types;
mod validate;

use csv::StringRecord;
use rayon::prelude::*;
use std::error::Error;
use std::io;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use state::State;
use types::{OutputRecord, TransactionRecord};

// TODO: Test locked account
// TODO: Test duplicate transaction id for valid first transaction
// TODO: Test duplicate transaction id for invalid first transaction
// TODO: Test unordered transaction ids
// TODO: Test dispute referencing nonexistent transaction
// TODO: Test resolve / chargeback referencing nonexistent transaction
// TODO: Test resolve / chargeback referencing undisputed transaction
// TODO: Test dispute / resolve / chargeback with client_id not matching referenced transaction

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

/// Can happen in any order
fn deserialize_record(record: StringRecord, headers: &StringRecord) -> Option<TransactionRecord> {
    match record.deserialize(Some(headers)) {
        Ok(ab) => Some(ab),
        Err(err) => {
            log::error!("Error while deserializing: {}", err);
            None
        }
    }
}

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

fn handle_transactions_on_thread(rcv: Receiver<TransactionRecord>, arc_state: Arc<State>) {
    rcv.iter()
        .map(|record| handlers::handle_transaction(record, arc_state));
}

/// Determine which thread should process the transaction.
///
/// Currently, just using client_id % num_threads,
/// since all clients are independent, so we need only
/// ensure that no two threads simultaneously handle
/// transactions on the same account.
fn assign_tx_to_thread(tx: TransactionRecord, num_threads: usize) -> usize {
    let thread_id = usize::from(tx.client_id) % num_threads;
    thread_id
}

fn spawn_handler_threads(
    num_threads: usize,
    arc_state: Arc<State>,
) -> (Vec<JoinHandle<()>>, Vec<SyncSender<TransactionRecord>>) {
    // TODO: State needs to be behind an Arc
    (0..num_threads)
        .map(|_| {
            // TODO: How large should the buffer be?
            let (snd, rcv) = sync_channel::<TransactionRecord>(10);
            let join_handle = thread::spawn(|| handle_transactions_on_thread(rcv, arc_state));
            (join_handle, snd)
        })
        .unzip()
}

fn handle_tx_batch(tx_batch: Vec<TransactionRecord>, snd: SyncSender<TransactionRecord>) {
    for record in tx_batch {
        if let Err(err) = snd.send(record) {
            log::error!("Error while sending transaction to worker: {}", err);
        }
    }
}

pub fn process_transactions<R: io::Read + Send + 'static, W: io::Write>(
    input_stream: R,
    output_stream: &mut W,
    batch_size: usize,
) {
    // TODO: Async / multithreaded?
    let mut arc_state = Arc::from(State::new());

    // Maximum number of batches to keep in the channel at once.
    // Once this limit is reached, IO will pause until one is processed.
    let max_batches = 1;

    let (records_snd, records_rcv) = sync_channel::<Vec<StringRecord>>(max_batches);
    let (headers_snd, headers_rcv) = sync_channel::<StringRecord>(1);

    let reader_handle = thread::spawn(move || {
        read_string_records(input_stream, headers_snd, records_snd, batch_size)
    });

    // TODO: CLI arg
    let num_threads = 4;
    let (handles, senders) = spawn_handler_threads(num_threads, arc_state.clone());

    if let Ok(headers) = headers_rcv.recv() {
        for record_batch in records_rcv {
            let tx_batch: Vec<_> = record_batch
                .into_par_iter()
                .filter_map(|record| deserialize_record(record, &headers))
                .collect();

            for tx in tx_batch {
                let thread_id = assign_tx_to_thread(tx, num_threads);
                let snd = senders[thread_id];
                if let Err(err) = snd.send(tx) {
                    log::error!("Failed to send transaction record to handler thread");
                }
            }
        }
    } else {
        log::error!("Failed to get CSV headers from reader thread");
    }

    write_balances(&arc_state, output_stream);

    // Should already have finished, but wait just in case
    if let Err(err) = reader_handle.join() {
        log::error!("Failed to join reader thread: {:?}", err);
    }
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
