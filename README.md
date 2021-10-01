[![Build Status](https://app.travis-ci.com/OliverEvans96/payments-engine-example.svg?branch=main)](https://app.travis-ci.com/OliverEvans96/payments-engine-example)
[![Docs](https://assets.readthedocs.org/static/projects/badges/passing-flat.svg)](https://oliverevans96.github.io/payments-engine-example/payments_engine_example/index.html)

<!-- [![Coverage Status](https://coveralls.io/repos/github/OliverEvans96/payments-engine-example/badge.svg?branch=main)](https://coveralls.io/github/OliverEvans96/payments-engine-example?branch=main) -->

# Payments Engine Example

```
payments-engine-example 0.1
Oliver Evans <oliverevans96@gmail.com>
Simple engine to process streaming financial transactions and write final account balances as output.

USAGE:
    payments-engine-example [FLAGS] [OPTIONS] <input-csv-path>

FLAGS:
    -h, --help       Prints help information
        --notrim     Disable trimming whitespace from CSV records. This can speed up deserialization significantly
    -V, --version    Prints version information

OPTIONS:
    -b <batch-size>                 Batch size for parallel CSV deserialization [default: 1000]
    -d <deserialize-workers>        Number of threads to dedicate to deserialization. Defaults to half of the system's
                                    logical cores

ARGS:
    <input-csv-path>    Path to transactions CSV file, or '-' for stdin
```


## Problem Overview

The prompt for this exercise is as follows:

We have many clients, each of whom have a single account.
Each account has three balances:
- `available` - amount of money available for withdrawal / use
- `held` - amount of disputed funds not currently available
- `total` - sum of `available` and `held`

There are five types of transactions:
- deposit - the client adds funds to their account.
- withdrawal - the client removes funds from their account.
- dispute - the client requests that a transaction be reversed. Previously available funds from the transaction become held.
- resolve - a dispute is settled, and the original transaction stands.
- chargeback - a dispute is settled, and the transaction is reversed. The client's account becomes frozen.

Note that disputes, resolves, and chargebacks don't have their own transaction ids, they only reference deposits and withdrawals.

Given a sequence of input transactions in CSV format, this program should write CSV records to `stdout` with the final state of all accounts.

Input CSVs (`transactions.csv`) look something like this:

```
type,        client,  tx,     amount
deposit,     1443,    1,      1216.5774
deposit,     2838,    2,      606.6812
dispute,     2842,    729,
withdrawal,  3607,    858,    3378.8557
chargeback,  6730,    1131,
withdrawal,  57,      9995,   11382.196
deposit,     91,      9996,   4189.032
resolve,     1302,    3383,
deposit,     20,      9997,   1914.785
dispute,     4317,    772,
chargeback,  6305,    2142,
resolve,     1577,    1581,
deposit,     18,      10000,  3938.4937
```

and output CSVs (`accounts.csv`) look like this:

```
client,  available,  held,  total,      locked
28,      12825.617,  0.0,   12825.617,  false
90,      2165.9717,  0.0,   2165.9717,  false
82,      20159.152,  0.0,   20159.152,  false
22,      4659.0273,  0.0,   4659.0273,  true
51,      2993.004,   0.0,   2993.004,   false
87,      25676.127,  0.0,   25676.127,  false
45,      3706.6443,  0.0,   3706.6443,  false
83,      26884.957,  0.0,   26884.957,  false
52,      4030.088,   0.0,   4030.088,   false
```


## Solution Overview

So let me tell you what I've done.


### Assumptions

Having very little knowledge of banking, the prompt inevitably leaves a bit of room for interpretation.
I've made the following assumptions:
- Deposits and withdrawals must have positive amounts.
- Once a transaction has been disputed and settled, it can't be re-disputed. Otherwise, you risk chargeback loops, which is certainly not desirable.
- Locked accounts cannot deposit or withdrawal, but can dispute, resolve and chargeback.
- *Only deposits can be disputed*. Given the instruction that disputes should _increase_ the `held` amount, I just haven't figured how that would make sense if disputing withdrawals were allowed.
- Negative balances are not impossible. If a deposit, withdrawal, dispute-deposit sequence yields a negative balance, it's our fault for approving the chargeback.


### Data Structures

The approach I'm taking is pretty straightforward.
I'm storing all application state in a single `State` struct, which has three fields: `accounts`, `transactions`, and `disputes`, each having type `AccountsState`, `TransactionsState`, and `DisputesState` respectively.

- `AccountsState` simply wraps a `HashMap` of `Account`s indexed by `client_id`.
- `TransactionsState` has a two parts:
    - a nested `HashMap` pair, indexing transactions by client, then by transacion id for transaction lookups
    - a `HashSet` of all transaction ids for duplicate identification
- `DisputesState` has two fields, both of which are `HashSets` of `tx_id`s nested inside of a `HashMap` keyed by `client_id`. One field is for actively disputed transaction ids, and the other is for previously disputed (settled) transactions.

Using outer `HashMaps` in these data structures to group by `client_id` is not strictly necessary, and I wasn't initially doing this, but it became necessary once I wanted to generate valid test transactions, and I thought it would eventually make parallelizing transaction processing simpler, since in the current paradigm, all accounts are independent, making for theoretically low-hanging parallelizable fruit.


### The Life of a Transaction

I'm using `serde` and the `csv` crate to deserialize each CSV lines into a `TransactionRecord` struct, which contains a `TransactionType` enum.
Then, I'm `match`ing on `TransactionType` to convert to a specific type of transaction (e.g. `Dispute`, `Withdrawal`), which implements the common `Transaction` trait.

That struct then gets validated, checking that both it's well-formatted and has sensible values, _and_ that it's a legal transaction considering current state of the engine based on all previous transactions.

The last step of each validation function is to return an `AccountAccess` enum, which gives _appropriate_ mutable access to the account in question based on its current state.
It may be of a `Locked` or `Unlocked` variant depending on the state of the account.
The `Locked` variant wraps a `LockedAccount` struct, and the `Unlocked` variant wraps an `UnlockedAccount`.
Both `LockedAccount` and `UnlockedAccount` implement the `BaseAccountFeatures` trait, which allow updating account balances for disputing, resolving, or charging-back previous transactions.
But only `UnlockedAccount` implements `UnlockedAccountFeatures`, which allows updating balances for new deposits and withdrawals, as well as locking the account.
Currently, the system has no concept of unlocking an account, but this could be achieved via a `LockedAccountFeatures` trait providing an `.unlock()` method, implemented only by `LockedAccount`. See `account.rs` for details.

Once the account has been updated, the transaction gets wrapped in a `TransactionContainer` enum with a variant for each relevant transaction type, and stored in the `state.transactions` HashMap for easy lookup down the road.

Currently, only withdrawals and deposits are being stored in `TransactionContainers`.
For now, it's just not necessary to store the other three, and they don't even have their own `tx_id`s.


### Extensibility

I mentioned above that only deposits are disputable based on my limited understanding of the scenario.
Presumably, everything should be disputable in the real world.
Luckily, if someone comes along who knows how to dispute another type of transaction, they simply need to implement the `Disputable` trait for that type, which specifies how to modify balances for disputes, resolves, and chargebacks.
They'll also need to "register" this new implementation by adding a `match` arm to the `try_get_disputable` function on `TransactionContainer`, which attempts to downcast a specific transaction type into `impl Disputable` if we know how to do so. See `traits.rs` for details.


### Maintainability

TODO

- returning early
- using ?
- controlled getter / setter methods, communicating via public interfaces
- type aliases: TransactionId, ClientId, CurrencyFloat


### Code Organization

TODO

- maintainability


## Automated testing

TODO

- testing
    - data-driven tests
    - inline tests
    - unit tests


## Generating Test Data

TODO

- transaction generation
    - process
    - data files

```
generate-transactions 0.1
Oliver Evans <oliverevans96@gmail.com>
Generate random valid transactions for payment processing engine.

USAGE:
    generate-transactions [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --attempts <attempts>            Maximum number of times to attempt to generate a new valid transaction before
                                         aborting [default: 10000]
    -c, --clients <clients>              Maximum number of clients to generate transactions for. Client IDs will be
                                         between 1 and this number [default: 100]
    -d, --deposit <deposit>              Maximum amount for deposits [default: 10000]
    -t, --transactions <transactions>    Number of transactions to generate. Defaults to infinite (run until cancelled)
```


## Performance & Efficiency

1bfde6d5 - 10 million in 13.95 seconds = 716k tx/sec
without trim - 10.87 sec = 920k tx/sec

4 des CPUS - 11.84 = 844k tx/sec
without trim - 10.85 sec = 921 tx/sec

TODO

- efficiency
    - profiling screenshots
    - rayon
    - performance
    - attempt to parallelize transaction handling
        - parallel across accounts
        - couldn't return references behind rwlock / mutex
            - owned_ref
            - parking_log
            - https://stackoverflow.com/questions/40095383/how-to-return-a-reference-to-a-sub-value-of-a-value-that-is-under-a-mutex


## Safety & Error Handling

TODO

- safety / error handling (logging)
    - logging
    - error types & propagation
    - occasional use of ?


## CI / CD

TODO

- travis CI, docs
