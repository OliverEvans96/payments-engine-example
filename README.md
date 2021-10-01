![Travis (.com)](https://img.shields.io/travis/com/OliverEvans96/payments-engine-example)
[![Docs](https://assets.readthedocs.org/static/projects/badges/passing-flat.svg)](https://oliverevans96.github.io/payments-engine-example/payments_engine_example/index.html)]

<!-- [![Coverage Status](https://coveralls.io/repos/github/OliverEvans96/payments-engine-example/badge.svg?branch=main)](https://coveralls.io/github/OliverEvans96/payments-engine-example?branch=main) -->

# Overview

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

TODO

- interesting notes
    - possible to have negative balance (chargeback after withdrawal)

### Assumptions

TODO

- document assumptions
    - no re-disputing
    - no negative transactions
    - locked accounts cannot deposit or withdrawal, but can dispute, resolve and chargeback

- how to dispute withdrawal

## Code Organization

TODO

- maintainability


# Using Types

TODO

- enforce invariants with type system
    - account access (lock / unlock)
    - Disputable (& how to implement more)
    - CLI with structopt / clap
    - dyn

# Automated testing

TODO

- testing
    - data-driven tests
    - inline tests
    - unit tests

# Generating Test Data

TODO

- transaction generation
    - process
    - data files

# Performance & Efficiency

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


# Safety & Error Handling

TODO

- safety / error handling (logging)
    - logging
    - error types & propagation
    - occasional use of ?

# CI / CD

TODO

- travis CI, docs
