use crate::account::Account;

#[derive(serde::Serialize)]
pub struct CsvRow {
    client: u16,
    available: String,
    held: String,
    total: String,
    locked: bool,
}

impl CsvRow {
    pub fn from_account(acc: Account) -> Self {
        // On debug mode, perform a sanity check before printing.
        debug_assert_eq!(acc.total, acc.available + acc.held);
        let available = acc.available as f64 / 10_000.0;
        let held = acc.held as f64 / 10_000.0;
        let total = acc.total as f64 / 10_000.0;
        Self {
            client: acc.client,
            available: format!("{available:.4}"),
            held: format!("{held:.4}"),
            total: format!("{total:.4}"),
            locked: acc.locked,
        }
    }
}
