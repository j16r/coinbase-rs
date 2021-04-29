use coinbase_rs::{Public, MAIN_URL};

#[tokio::main]
async fn main() {
    let client: Public = Public::new(MAIN_URL);

    for currency in client.currencies().await.unwrap() {
        println!(
            "Currency {} mininum size = {}",
            currency.name, currency.min_size
        );
    }
}
