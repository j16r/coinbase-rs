use coinbase_rs::{Public, MAIN_URL};
use futures::pin_mut;
use futures::stream::StreamExt;

#[tokio::main]
async fn main() {
    let client = Public::new(MAIN_URL);

    let currencies = client.currencies();
    pin_mut!(currencies);

    while let Some(currencies_result) = currencies.next().await {
        for currency in currencies_result.unwrap() {
            println!(
                "Currency {} mininum size = {}",
                currency.name, currency.min_size
            );
        }
    }
}
