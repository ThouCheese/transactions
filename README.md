Hello dear reviewer. In this readme you will find the table of scoring criteria and the
explainations for them, as per request from the exercise file. Enjoy!

### Basics
The applications builds with the Rust 1.61.0 toolchain. I did some destructuring assignments for
conciseness, so I added a rust-toolchain file, specifying that cargo should run with version 1.61.0.
The code is formatted with the default settings of rustfmt.

### Completness
I was able to handle all types of transactions.

### Correctness
I did some work on type-level correctness, i.e. using a sperate type for a Mutation and a
Transaction, where each Transaction is guaranteed to have an amount, whereas Mutations are allowed
to not have an amount. Furthermore I used a wrapper struct for the Transactions and Accounts state
that is maintained.

I also wrote some tests for the happy flows (and a little bit of the sad flows) in the code that
modifies the account balances (account.rs), but I did not test all of the unhappy paths because this
code will not go into production and the three hours were up.

### Safety and Robustness
There is no usage of `unsafe` anywhere, `unwrap` is restricted to tests and all of the error
handling is graceful through `Result`. I used `eyre` for convenient error handling.

### Efficiency
The program avoids having to load the whole csv into memory at once, simply iterating over the rows
of the csv, but I was not able to avoid keeping all of the Deposits and Withdrawals in memory. The
reason for this is that the other 3 types of transaction reference the deposit they are related to
by id, so we need to do a lookup. I kept the Withdrawals in memory to present a nice error message
when someone tries to initiate the refund flow on a Withdrawal.

> What if your code was bundled in a server, and these CSVs came from thousands of concurrent TCP
> streams?

This should not be a problem, but in order to make multi threading possible we'd need to switch over
to a concurrent map implementation like [DashMap](https://docs.rs/dashmap).

### Maintainability
The code is reasonably well-documented and in the parsing and presenting sections I have chose not
to go to crazy with serde field attributes to make the structs get displayed correctly, because this
requires some knowledge about the serde data model and Serializers/Deserializers. Instead I have
created a seperate struct that just has the fields we need as correctly formatted strings.
