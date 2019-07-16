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
