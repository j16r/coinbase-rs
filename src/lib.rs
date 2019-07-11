#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
extern crate base64;
extern crate hmac;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;
extern crate sha2;
extern crate tokio;

pub mod adapters;
pub mod error;
pub mod private;
pub mod public;

pub use adapters::Sync;
pub use error::CBError;
pub use private::Private;
pub use public::Public;

pub const MAIN_URL: &str = "https://api.coinbase.com/v2";
