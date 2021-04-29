use coinbase_rs::{Private, MAIN_URL};
use std::str::FromStr;
use uuid::Uuid;

pub const KEY: &str = "<put key here>";
pub const SECRET: &str = "<put secret here>";

#[tokio::main]
async fn main() {
    let client: Private = Private::new(MAIN_URL, KEY, SECRET);

    let accounts = client.accounts().await.unwrap();
    for account in accounts {
        println!("Account {}", account.currency.code);
        if let Ok(id) = Uuid::from_str(&account.id) {
            for transaction in client.transactions(&id).await.unwrap() {
                println!(
                    "Transaction {} = {}",
                    transaction.id, transaction.amount.amount
                );
            }
        }
    }
}
