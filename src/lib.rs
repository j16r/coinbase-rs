#[macro_use]
extern crate serde_derive;
extern crate failure;
extern crate futures;
extern crate base64;
extern crate hmac;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;
extern crate sha2;
extern crate tokio;
extern crate tokio_stream;
extern crate uritemplate;

pub mod adapters;
pub mod error;
pub mod private;
pub mod public;

pub use adapters::Sync;
pub use error::CBError;
pub use private::Private;
pub use public::Public;

pub const MAIN_URL: &str = "https://api.coinbase.com/v2";

pub type DateTime = chrono::DateTime<chrono::Utc>;
