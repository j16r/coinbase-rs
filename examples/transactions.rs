use coinbase_rs::{Private, Uuid, MAIN_URL};
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
