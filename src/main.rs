use std::fs;
use std::io;
use structopt::StructOpt;

use payments_engine_example::{configure_deserialize_workers, process_transactions};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "payments-engine-example",
    version = "0.1",
    author = "Oliver Evans <oliverevans96@gmail.com>",
    about = "Simple engine to process streaming financial transactions and write final account balances as output."
)]
struct CliOpts {
    /// Path to transactions CSV file, or '-' for stdin
    input_csv_path: String,

    /// Batch size for parallel CSV deserialization.
    #[structopt(short, default_value = "1000")]
    batch_size: usize,

    /// Number of threads to dedicate to deserialization.
    /// Defaults to half of the system's logical cores.
    #[structopt(short)]
    deserialize_workers: Option<usize>,
}

fn main_command(path: &str, batch_size: usize) {
    // Write to stdout
    let mut output = io::stdout();

    // Read from stdin or file
    if path == "-" {
        let input = io::stdin();
        process_transactions(input, &mut output, batch_size);
    } else {
        if let Ok(input) = fs::File::open(&path) {
            process_transactions(input, &mut output, batch_size);
        } else {
            log::error!("Could not open input file '{}'", &path);
        }
    }
}

fn main() {
    // Allow log level to be set via env vars without recompiling
    env_logger::init();

    // Parse arguments
    let CliOpts {
        input_csv_path,
        batch_size,
        deserialize_workers,
    } = CliOpts::from_args();

    // Configure rayon thread pool
    configure_deserialize_workers(deserialize_workers);

    // Run
    main_command(&input_csv_path, batch_size);
}
