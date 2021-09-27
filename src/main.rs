use std::env;
use std::error::Error;
use std::fs;
use std::io;

use payments_engine_example::process_transactions;

// TODO: CL args
fn main() -> Result<(), Box<dyn Error>> {
    // Allow log level to be set via env vars without recompiling
    env_logger::init();

    // First arg is path to executable, not important
    let mut args = env::args().skip(1);

    // TODO: Clean up CLI, add help, etc.
    let input_csv_path = args
        .next()
        .expect("Missing required command line argument - input csv path");

    // Check for extraneous arguments
    if args.len() > 0 {
        log::warn!(
            "unused command line arguments: {:?}",
            args.collect::<Vec<_>>()
        );
    }

    // Open file and process transactions, writing to stdout
    if let Ok(mut input_file) = fs::File::open(&input_csv_path) {
        // TODO: Handle errors
        process_transactions(&mut input_file, &mut io::stdout())?;
    } else {
        log::error!("Could not open input file '{}'", input_csv_path);
    }

    // :)
    Ok(())
}

#[cfg(test)]
mod tests {
    // TODO: unit tests
}
