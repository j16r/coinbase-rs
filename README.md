# Coinbase Client for Rust

[![Actions Status](https://github.com/j16r/coinbase-rs/workflows/Rust/badge.svg)](https://github.com/j16r/coinbase-rs/actions)

This is a Rust client for interacting with the Coinbase API. It
works with version 2019-04-03 (v2) of the API. This is not compatible with the
Coinbase Pro API.

## Features

- Private and Public API
- Pagination through streams

## Examples

Cargo.toml:

```toml
[dependencies]
coinbase-rs = "0.3.0"
```

### Public API

```rust
use coinbase_rs::{Public, MAIN_URL};

#[tokio::main]
async fn main() {
    let client = Public::new(MAIN_URL);
    println!("Server time is {:?}", client.current_time().await.unwrap());
}
```

### Private API

```rust
use coinbase_rs::{Private, MAIN_URL, Uuid};
use futures::pin_mut;
use futures::stream::StreamExt;
use std::str::FromStr;

pub const KEY: &str = "<put key here>";
pub const SECRET: &str = "<put secret here>";

#[tokio::main]
async fn main() {
    let client = Private::new(MAIN_URL, KEY, SECRET);

    let accounts = client.accounts();
    pin_mut!(accounts);

    while let Some(account_result) = accounts.next().await {
        for account in account_result.unwrap() {
            println!("Account {}", account.currency.code);
            if let Ok(id) = Uuid::from_str(&account.id) {
                let transactions = client.transactions(&id);
                pin_mut!(transactions);

                while let Some(transactions_result) = transactions.next().await {
                    for transaction in transactions_result.unwrap() {
                        println!(
                            "Transaction {} = {}",
                            transaction.id, transaction.amount.amount
                        );
                    }
                }
            }
        }
    }
}
```

## Thanks

This project is inspired and borrows heavily from
[coinbase-pro-rs](https://github.com/inv2004/coinbase-pro-rs).
