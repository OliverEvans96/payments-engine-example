use std::fs;
use std::io;
use std::process::exit;

use payments_engine_example::process_transactions;
use payments_engine_example::rand::generate_random_valid_transaction_sequence;
use payments_engine_example::types::{ClientId, CurrencyFloat, TransactionId};
use structopt::StructOpt;

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
    #[structopt(subcommand)]
    cmd: Option<Subcommand>,
}

#[derive(Debug, StructOpt)]
enum Subcommand {
    /// Generate random transaction data to feed to the engine
    GenerateTransactions {
        /// Number of transactions to generate.
        /// Defaults to infinite (run until cancelled)
        #[structopt(short, long)]
        transactions: Option<TransactionId>,

        /// Maximum number of clients to generate transactions for.
        /// Client IDs will be between 1 and this number.
        #[structopt(short, long, default_value = "100")]
        clients: ClientId,

        /// Maximum amount for deposits.
        #[structopt(short, long, default_value = "10000")]
        amount: CurrencyFloat,
    },
}

fn main_command(path: &str) {
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

fn generate_transactions(
    num_tx: Option<TransactionId>,
    max_client: ClientId,
    max_amount: CurrencyFloat,
) {
    // Write to stdout
    let output = io::stdout();
    let mut writer = csv::Writer::from_writer(output);

    let seq = generate_random_valid_transaction_sequence(num_tx, max_client, max_amount);
    let mut num_generated = 0;
    for tx in seq {
        if let Err(err) = writer.serialize(tx) {
            log::error!("Error writing generated transaction: {}", err);
        } else {
            num_generated += 1;
        }
    }

    if let Some(desired) = num_tx {
        if num_generated < desired {
            log::error!(
                "Only generated {} / {} transactions.",
                num_generated,
                desired
            );
            exit(1);
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
            Subcommand::GenerateTransactions {
                clients,
                transactions,
                amount,
            } => generate_transactions(transactions, clients, amount),
        }
    } else {
        main_command(&opts.input_csv_path)
    }
}
