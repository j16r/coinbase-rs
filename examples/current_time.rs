use coinbase_rs::{Public, MAIN_URL};

#[tokio::main]
async fn main() {
    let client = Public::new(MAIN_URL);
    println!("Server time is {:?}", client.current_time().await.unwrap());
}
