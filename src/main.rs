use std::fs;
use std::io;
use std::path;

use structopt::StructOpt;

use payments_engine_example::process_transactions;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "payments-engine-example",
    version = "0.1",
    author = "Oliver Evans <oliverevans96@gmail.com>",
    about = "Simple engine to process streaming financial transactions and write final account balances as output"
)]
struct CliOpts {
    /// Path to transactions CSV file
    #[structopt(parse(from_os_str))]
    input_csv_path: path::PathBuf,
}

fn main() {
    // Allow log level to be set via env vars without recompiling
    env_logger::init();

    let opts = CliOpts::from_args();

    // Open file and process transactions, writing to stdout
    if let Ok(mut input_file) = fs::File::open(&opts.input_csv_path) {
        process_transactions(&mut input_file, &mut io::stdout());
    } else {
        log::error!(
            "Could not open input file '{}'",
            &opts.input_csv_path.to_str().unwrap_or("<invalid path>")
        );
    }
}