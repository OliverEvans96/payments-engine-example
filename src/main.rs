use std::fs;
use std::io;
use rand::Rng;

use payments_engine_example::types::TransactionRecord;
use structopt::StructOpt;
use payments_engine_example::process_transactions;
use structopt::clap::SubCommand;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "payments-engine-example",
    version = "0.1",
    author = "Oliver Evans <oliverevans96@gmail.com>",
    about = "Simple engine to process streaming financial transactions and write final account balances as output"
)]
struct CliOpts {
    /// Path to transactions CSV file, or '-' for stdin
    input_csv_path: String,

    /// Optional subcommands are not required
    cmd: Option<Subcommand>,
}

#[derive(Debug, StructOpt)]
enum Subcommand {
    /// Generate random transaction data to feed to the engine
    GenerateTransactions {
        /// Number of transactions to generate.
        /// Defaults to infinite (run until cancelled).
        #[structopt(short, long)]
        transactions: Option<u32>,

        /// Maximum number of clients to generate transactions for.
        /// Client IDs will be between 1 and this number.
        #[structopt(short, long)]
        clients: Option<u32>,
    },
}

fn handle_main_command(path: &str) {
    // Write to stdout
    let mut output = io::stdout();

    // Read from stdin or file
    if path == "-" {
        let mut input = io::stdin();
        process_transactions(&mut input, &mut output);
    } else {
        if let Ok(mut input) = fs::File::open(&path) {
            process_transactions(&mut input, &mut output);
        } else {
            log::error!("Could not open input file '{}'", &path);
        }
    }
}

fn generate_transactions(number: Option<u32>) {
    // Write to stdout
    let mut output = io::stdout();
    let writer = csv::Writer::from_writer(output);
    let rng = rand::thread_rng();

    if let Some(max) = number {
        for _ in 0..max {
            let record: TransactionRecord = rng.gen();
            if let Err(err) = writer.serialize(record) {
                log::error!("Error writing generated transaction: {}", err);
            }
        }
    } else {
        loop {

        }
    }
}

fn main() {
    // Allow log level to be set via env vars without recompiling
    env_logger::init();

    // Parse arguments
    let opts = CliOpts::from_args();

    if let Some(subcommand) = opts.cmd {
        match subcommand {
            SubCommand::GenerateTransactions { number } => generate_transactions(number),
        }
    } else {
        main_command(opts.input_csv_path)
    }
}
