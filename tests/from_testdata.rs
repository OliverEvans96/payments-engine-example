use payments_engine_example::process_transactions;
use payments_engine_example::types::OutputRecord;
use std::error::Error;
use std::fs;
use std::io;
use std::path;

fn run_test_from_directory(directory: path::PathBuf) -> Result<(), Box<dyn Error>> {
    let transactions_path = directory.join("transactions.csv");
    let accounts_path = directory.join("accounts.csv");

    let transactions_file = fs::File::open(&transactions_path).expect(&format!(
        "Failed to open transactions file '{}'",
        transactions_path.to_str().unwrap_or("<invalid path>")
    ));

    // Write results to in-memory buffer
    let mut output_buf = io::Cursor::new(Vec::new());
    process_transactions(transactions_file, &mut output_buf);

    // Re-deserialize actual results from output buffer
    output_buf.set_position(0);
    let actual_accounts_reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(&mut output_buf);

    // Read expected results from file
    let expected_accounts_reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(&accounts_path)
        .expect(&format!(
            "Failed to open accounts file '{}'",
            accounts_path.to_str().unwrap_or("<invalid path>")
        ));

    // Be reckless: serialize whole files into memory, failing if any error is encountered
    let mut expected_accounts: Vec<OutputRecord> = expected_accounts_reader
        .into_deserialize()
        .collect::<Result<Vec<_>, _>>()?;
    let mut actual_accounts: Vec<OutputRecord> = actual_accounts_reader
        .into_deserialize()
        .collect::<Result<Vec<_>, _>>()?;

    // Sort values by client id before comparing since the order of rows is not significant
    expected_accounts.sort_by_key(|rec| rec.client);
    actual_accounts.sort_by_key(|rec| rec.client);

    assert_eq!(
        expected_accounts,
        actual_accounts,
        "test failure in {:?}",
        directory.to_str().unwrap_or("<invalid path>")
    );

    Ok(())
}

#[test]
fn run_tests_from_testdata() -> Result<(), Box<dyn Error>> {
    let testdata_path = path::Path::new("testdata");

    for directory in fs::read_dir(testdata_path).unwrap() {
        let test_path = directory.unwrap().path();
        println!(
            "Running test from directory: {}",
            test_path.to_str().unwrap_or("<invalid path>")
        );
        run_test_from_directory(test_path)?;
    }

    Ok(())
}
