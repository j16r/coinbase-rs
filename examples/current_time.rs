use coinbase_rs::{Public, Sync, MAIN_URL};

fn main() {
    let client: Public<Sync> = Public::new(MAIN_URL);
    println!("Server time is {}", client.current_time().unwrap());
}
