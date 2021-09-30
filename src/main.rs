use std::fs;
use std::io;
use structopt::StructOpt;

use payments_engine_example::process_transactions;

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
}

fn main_command(path: &str) {
    // Write to stdout
    let mut output = io::stdout();

    // Read from stdin or file
    if path == "-" {
        let input = io::stdin();
        process_transactions(input, &mut output);
    } else {
        if let Ok(input) = fs::File::open(&path) {
            process_transactions(input, &mut output);
        } else {
            log::error!("Could not open input file '{}'", &path);
        }
    }
}

fn main() {
    // Allow log level to be set via env vars without recompiling
    env_logger::init();

    // Parse arguments
    let opts = CliOpts::from_args();

    // Run
    main_command(&opts.input_csv_path);
}
