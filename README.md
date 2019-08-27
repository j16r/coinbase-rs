# Coinbase Client for Rust

[![Actions Status](https://github.com/j16r/coinbase-rs/workflows/Rust/badge.svg)](https://github.com/j16r/coinbase-rs/actions)

This is a Rust client for interacting with the Coinbase API. It
works with version 2019-04-03 (v2) of the API. This is not compatible with the
Coinbase Pro API.

## Features

- Sync and Async support
- Private and Public API

## Examples

Cargo.toml:

```toml
[dependencies]
coinbase-rs = "0.2.0"
```

### Public API (Sync)

```rust
use coinbase_rs::{Public, Sync, MAIN_URL};

fn main() {
    let client: Public<Sync> = Public::new(MAIN_URL);

    for currency in client.currencies().unwrap() {
        println!(
            "Currency {} mininum size = {}",
            currency.name, currency.min_size
        );
    }
}
```

### Private API (Sync)

```rust
use coinbase_rs::{Private, Sync, MAIN_URL};
use std::str::FromStr;
use uuid::Uuid;

pub const KEY: &str = "<put key here>";
pub const SECRET: &str = "<put secret here>";

fn main() {
    let client: Private<Sync> = Private::new(MAIN_URL, KEY, SECRET);

    let accounts = client.accounts().unwrap();
    for account in accounts {
        println!("Account {}", account.currency.code);
        if let Ok(id) = Uuid::from_str(&account.id) {
            for transaction in client.list_transactions(&id).unwrap() {
                println!(
                    "Transaction {} = {}",
                    transaction.id, transaction.amount.amount
                );
            }
        }
    }
}
```

## Thanks

This project is inspired and borrows heavily from
[coinbase-pro-rs](https://github.com/inv2004/coinbase-pro-rs).
