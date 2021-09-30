use std::io;
use std::process::exit;
use structopt::StructOpt;

use payments_engine_example::rand::generate_random_valid_transaction_sequence;
use payments_engine_example::types::{ClientId, CurrencyFloat, TransactionId};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "generate-transactions",
    version = "0.1",
    author = "Oliver Evans <oliverevans96@gmail.com>",
    about = "Generate random valid transactions for payment processing engine."
)]
struct CliOpts {
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
    deposit: CurrencyFloat,

    /// Maximum number of times to attempt to generate
    /// a new valid transaction before aborting
    #[structopt(short, long, default_value = "10000")]
    attempts: usize,
}

fn generate_transactions(
    num_tx: Option<TransactionId>,
    max_client: ClientId,
    max_deposit: CurrencyFloat,
    max_attempts: usize,
) {
    // Write to stdout
    let output = io::stdout();
    let mut writer = csv::Writer::from_writer(output);

    let seq =
        generate_random_valid_transaction_sequence(num_tx, max_client, max_deposit, max_attempts);
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
    let CliOpts {
        clients,
        transactions,
        deposit,
        attempts,
    } = CliOpts::from_args();

    generate_transactions(transactions, clients, deposit, attempts);
}
