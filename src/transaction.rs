use eyre::{eyre, Result};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

/// A full collection of all transactions that we have visisted so far. It is sad that we need to
/// maintain this data, but since Disputes, Resolves and Chargebacks do not actually contain
/// information about the amounts that are involved, we are forced to. This facilitates looking up
/// the previously ingested transaction by the transaction id.
#[derive(Default)]
pub struct Transactions {
    /// A map from transaction id to the amount that that transaction contained. We use a HashMap
    /// because we need to do many random lookups by id, so this gets us O(1) time for that
    /// operation.
    trxs: HashMap<u32, Transaction>,
}

/// We allow our dataset to be accessed as though it were a specially typed HashMap. For this reason
/// we implement Deref and DerefMut for `Transactions`.
impl Deref for Transactions {
    type Target = HashMap<u32, Transaction>;

    fn deref(&self) -> &Self::Target {
        &self.trxs
    }
}

impl DerefMut for Transactions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.trxs
    }
}

/// A transaction that has been performed.
#[derive(Debug, PartialEq, Eq)]
pub struct Transaction {
    pub id: u32,
    pub kind: TransactionType,
    pub client: u16,
    pub amount: u32,
    pub status: TransactionStatus,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Mutation {
    pub id: u32,
    pub kind: TransactionType,
    pub client: u16,
    pub amount: Option<u32>,
}

impl TryInto<Transaction> for Mutation {
    type Error = eyre::Report;

    fn try_into(self) -> Result<Transaction> {
        let id = self.id;
        // In our parsing logic we have made sure that this should never happen, but this sanity
        // check is still worthwhile, because someone could remove the verification that happens
        // during parsing.
        let err = || eyre!("Err for trx {id}, using transactions require an amount!");
        let trx = Transaction {
            id,
            kind: self.kind,
            client: self.client,
            amount: self.amount.ok_or_else(err)?,
            status: TransactionStatus::Ok,
        };
        Ok(trx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    Ok,
    Disputed,
    Resolved,
    Refunded,
}
