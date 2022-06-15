use crate::transaction::{Mutation, TransactionType};

/// This struct is meant only to parse rows from the input CSV. Since we need to track additional
/// data, we use a separate internal model ([Transaction](crate::transaction::Transaction)) to
/// operate on. We could implement Deserialize directly onto that model, but it would involve custom
/// deserialization through `#[serde(deserialize_with = "blabla")]`, so the more readable option of
/// a plain-rust struct-to-struct conversion is chosen for readability.
#[derive(serde::Deserialize)]
pub struct CsvRow {
    /// There are multiple transaction types, this field indicates which one this is. It is called
    /// kind because `type` is a reserved keyword.
    #[serde(rename = "type")]
    kind: TransactionType,
    /// The unique id of the client performing this transaction.
    client: u16,
    /// The unique id of the transaction being performed. Note that this uniquely identifies a
    /// transaction, but there may be multiple CSV rows per transaction as it moves through the
    /// stages of refunding.
    tx: u32,
    /// The amount of currency that is concerned.
    amount: Option<f64>,
}

impl CsvRow {
    /// The silent invariant for our program to operate in a sensible way is that fundamentally,
    /// deposits and withdrawals have an amount, whereas disputes, resolves and chargebacks do not.
    /// We perform a check here to make sure that we do not accidentally handle data in an
    /// unexpected way, and this is the reason that converting a CsvRow to a Mutation may fail.
    pub fn as_mutation(self) -> eyre::Result<Mutation> {
        use TransactionType::*;
        let err = |msg| Err(eyre::eyre!("Error parsing transaction {}, {msg}", self.tx));
        match (self.kind, self.amount) {
            (Deposit, None) => return err("deposits musts have an amount"),
            (Withdrawal, None) => return err("withdrawals must have an amount"),
            (Dispute, Some(_)) => return err("disputes may not have an amount"),
            (Resolve, Some(_)) => return err("resolves may not have an amount"),
            (Chargeback, Some(_)) => return err("chargebacks may not have an amount"),
            _ => {}
        };
        Ok(Mutation {
            id: self.tx,
            kind: self.kind,
            client: self.client,
            amount: self.amount.map(|a| (a * 10_000.0) as u32),
        })
    }
}
