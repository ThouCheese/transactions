use crate::transaction::{Mutation, Transaction, TransactionStatus, TransactionType, Transactions};
use eyre::{eyre, Result};
use std::collections::HashMap;

/// A collection of all the accounts we have accumulated so far, indexable by account id.
#[derive(Default)]
pub struct Accounts {
    /// A map from account id to the account info struct.
    accounts: HashMap<u16, Account>,
}

impl Accounts {
    pub fn account_for_id(&mut self, client: u16) -> &mut Account {
        self.accounts.entry(client).or_insert(Account::new(client))
    }
}

impl IntoIterator for Accounts {
    type Item = Account;

    type IntoIter = std::collections::hash_map::IntoValues<u16, Account>;

    fn into_iter(self) -> Self::IntoIter {
        self.accounts.into_values()
    }
}

/// A users account state. Since we are working with money, we do not store amounts as floats, but
/// rather we store the amount of smallest possible increments as an unsigned integer. This amount
/// if 0.0001 currency, since we are expected to maintain a precision of 4 decimals.
pub struct Account {
    pub client: u16,
    /// The amount of (currency * 10_000) available for trading and withdrawing.
    pub available: u32,
    /// The amount of (currency * 10_000) that is locked due to disputed transactions.
    pub held: u32,
    /// The amount of currency, expressed in f
    pub total: u32,
    pub locked: bool,
}

impl Account {
    pub fn new(client: u16) -> Self {
        Self {
            client,
            available: 0,
            held: 0,
            total: 0,
            locked: false,
        }
    }

    /// Mutates an account
    pub fn mutate(&mut self, trx: Mutation, trxs: &mut Transactions) -> Result<()> {
        if self.locked {
            let err = eyre!("Attempt to mutate account {}, which is locked", self.client);
            return Err(err);
        }
        match trx.kind {
            TransactionType::Deposit => self.process_deposit(trx, trxs),
            TransactionType::Withdrawal => self.process_withdrawal(trx, trxs),
            TransactionType::Dispute => self.process_dispute(trx.id, trxs),
            TransactionType::Resolve => self.process_resolve(trx.id, trxs),
            TransactionType::Chargeback => self.process_chargeback(trx.id, trxs),
        }
    }

    fn process_deposit(&mut self, trx: Mutation, trxs: &mut Transactions) -> Result<()> {
        let trx: Transaction = trx.try_into()?;
        self.available += trx.amount;
        self.total += trx.amount;
        trxs.insert(trx.id, trx);
        Ok(())
    }

    fn process_withdrawal(&mut self, trx: Mutation, trxs: &mut Transactions) -> Result<()> {
        let trx: Transaction = trx.try_into()?;
        let id = trx.id;
        let err = || {
            let amount = trx.amount as f64 / 10_000.0;
            eyre!("Error on trx {id}: Can't withdraw {amount}")
        };
        let available = self.available.checked_sub(trx.amount).ok_or_else(err)?;
        let total = self.total.checked_sub(trx.amount).ok_or_else(err)?;
        (self.available, self.total) = (available, total);
        trxs.insert(id, trx.try_into()?);
        Ok(())
    }

    fn process_dispute(&mut self, id: u32, trxs: &mut Transactions) -> Result<()> {
        let trx = match trxs.get_mut(&id) {
            Some(trx) if trx.status == TransactionStatus::Ok => trx,
            Some(trx) if trx.kind != TransactionType::Deposit => {
                return Err(eyre!("Cannot dispute {id}, only deposits can be disputed"));
            }
            // Trx doesnt exist or is not Ok, assume this is an error on our partners side.
            _ => return Ok(()),
        };
        let err = || {
            let amount = trx.amount as f64 / 10_000.0;
            eyre!("Error on trx {id}: Can't dispute {amount}")
        };
        self.available = self.available.checked_sub(trx.amount).ok_or_else(err)?;
        self.held += trx.amount;
        trx.status = TransactionStatus::Disputed;
        Ok(())
    }

    fn process_resolve(&mut self, id: u32, trxs: &mut Transactions) -> Result<()> {
        let trx = match trxs.get_mut(&id) {
            Some(trx) if trx.status == TransactionStatus::Disputed => trx,
            // Trx doesnt exist or is not Disputed, assume this is an error on our partners side.
            _ => return Ok(()),
        };
        let err = || {
            let amount = trx.amount as f64 / 10_000.0;
            eyre!("Error on trx {id}: Can't resolve {amount}")
        };
        self.available += trx.amount;
        self.held = self.held.checked_sub(trx.amount).ok_or_else(err)?;
        trx.status = TransactionStatus::Resolved;
        Ok(())
    }

    fn process_chargeback(&mut self, id: u32, trxs: &mut Transactions) -> Result<()> {
        let trx = match trxs.get_mut(&id) {
            Some(trx) if trx.status == TransactionStatus::Resolved => trx,
            // Trx doesnt exist or is not Resolved, assume this is an error on our partners side.
            _ => return Ok(()),
        };
        let err = || {
            let amount = trx.amount as f64 / 10_000.0;
            eyre!("Error on trx {id}: Can't chargeback {amount}")
        };
        let available = self.available.checked_sub(trx.amount).ok_or_else(err)?;
        let total = self.total.checked_sub(trx.amount).ok_or_else(err)?;
        (self.available, self.total) = (available, total);
        self.locked = true;
        trx.status = TransactionStatus::Refunded;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use TransactionType::*;

    fn mutation(id: u32, kind: TransactionType) -> Mutation {
        Mutation {
            id,
            kind,
            client: 1,
            amount: Some(5),
        }
    }

    #[test]
    fn test_deposit() {
        let mut account = Account::new(1);
        let mut trxs = Transactions::default();

        account
            .process_deposit(mutation(1, Deposit), &mut trxs)
            .unwrap();
        account.available = 5;
        account.held = 0;
        account.total = 5;
    }

    #[test]
    fn test_withdrawal() {
        let mut account = Account {
            client: 1,
            available: 7,
            held: 0,
            total: 7,
            locked: false,
        };
        let mut trxs = Transactions::default();

        account
            .process_withdrawal(mutation(1, Withdrawal), &mut trxs)
            .unwrap();
        account.available = 2;
        account.held = 0;
        account.total = 2;
        let withdraw2 = account.process_withdrawal(mutation(1, Withdrawal), &mut trxs);
        assert!(withdraw2.is_err());
    }

    #[test]
    fn test_dispute() {
        let mut account = Account::new(1);
        let mut trxs = Transactions::default();
        account.mutate(mutation(1, Deposit), &mut trxs).unwrap();

        account.process_dispute(1, &mut trxs).unwrap();
        assert_eq!(account.available, 0);
        assert_eq!(account.held, 5);
        assert_eq!(account.total, 5);
        // Disputing again must not error, we ignore this case.
        account.process_dispute(1, &mut trxs).unwrap();
        assert_eq!(account.available, 0);
        assert_eq!(account.held, 5);
        assert_eq!(account.total, 5);
    }

    #[test]
    fn test_resolve() {
        let mut account = Account::new(1);
        let mut trxs = Transactions::default();
        account.mutate(mutation(1, Deposit), &mut trxs).unwrap();
        account.mutate(mutation(1, Dispute), &mut trxs).unwrap();

        account.process_resolve(1, &mut trxs).unwrap();
        assert_eq!(account.available, 5);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 5);
        // Disputing again must not error, we ignore this case.
        account.process_resolve(1, &mut trxs).unwrap();
        assert_eq!(account.available, 5);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 5);
    }

    #[test]
    fn test_chargeback() {
        let mut account = Account::new(1);
        let mut trxs = Transactions::default();
        account.mutate(mutation(1, Deposit), &mut trxs).unwrap();
        account.mutate(mutation(1, Dispute), &mut trxs).unwrap();
        account.mutate(mutation(1, Resolve), &mut trxs).unwrap();

        account.process_chargeback(1, &mut trxs).unwrap();
        assert_eq!(account.available, 0);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 0);
        // Disputing again must not error, we ignore this case.
        account.process_chargeback(1, &mut trxs).unwrap();
        assert_eq!(account.available, 0);
        assert_eq!(account.held, 0);
        assert_eq!(account.total, 0);
    }
}
