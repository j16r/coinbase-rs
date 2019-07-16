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
