use std::io;

use crate::types::{OutputRecord, State};

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
