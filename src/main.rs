/// Contains the `Account` and `Accounts` structs that store the created list of accounts and their
/// balances.
mod account;
/// Contains the functionality needed to read the input CSV and map it to a useful struct.
mod parse;
/// Contains the functionality needed to display an output CSV, created from our internal data
/// structures.
mod present;
/// Contains the `Transaction` and `Transactions` structs that represent the flow of money into and
/// out of our accounts.
mod transaction;

use eyre::{eyre, Result};
use std::{
    fs,
    process::{ExitCode, Termination},
};

#[repr(u8)]
pub enum Exit {
    Success = 0,
    Failure = 1,
}

impl Termination for Exit {
    fn report(self) -> ExitCode {
        ExitCode::from(self as u8)
    }
}

/// Our main function offloads to `try_main`, and it itself is only concerned with exit codes and
/// displaying an eventual failure.
fn main() -> Exit {
    match try_main() {
        Ok(_) => Exit::Success,
        Err(msg) => {
            eprintln!("The transaction engine failed with message:\n{msg}");
            Exit::Failure
        }
    }
}

/// The meat of our application. Reads csv data from a csv (indicated by the first arg) and runs it
/// trough the engine to construct a list of accounts and transactions, then outputs the resulting
/// account states to stdout.
fn try_main() -> Result<()> {
    // Get a csv reader for the indicated file.
    let mut reader = reader()?;

    // Our state is maintained in these two structs, one contains all the accounts, whereas the
    // other contains a list of all deposited transactions.
    let mut accounts = account::Accounts::default();
    // This data structure will hold all of our transaction state, that is, deposits and
    // withdrawals. We would have preferred to not need to keep track of this, but since disputes,
    // resolves and chargebacks don't contain their own amount, we need to be able to look back at
    // the entire history of deposits and withdrawals.
    let mut trxs = transaction::Transactions::default();

    // We iterate over each record in the csv file.
    for result in reader.deserialize() {
        let record: parse::CsvRow = result?;

        let trx = record.as_mutation()?;
        // Get the correct account, and mutate it according to this transaction.
        accounts.account_for_id(trx.client).mutate(trx, &mut trxs)?;
    }

    // Now we are ready to print our data to stdout.
    let stdout = std::io::stdout().lock();
    let mut writer = csv::Writer::from_writer(stdout);
    for account in accounts {
        // We transform each account from our internal sturct to a struct that matches the csv rows
        // we need to produce.
        writer.serialize(present::CsvRow::from_account(account))?;
    }

    Ok(())
}

fn reader() -> Result<csv::Reader<fs::File>> {
    let name = std::env::args()
        .nth(1)
        .ok_or_else(|| eyre!("Usage: cargo run -- [input file].csv > [output file].csv"))?;
    let reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(name)?;
    Ok(reader)
}
