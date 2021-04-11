#![feature(async_stream)]
#![feature(min_type_alias_impl_trait)]

extern crate base64;
extern crate failure;
extern crate futures;
extern crate hmac;
extern crate http;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sha2;
extern crate tokio;
extern crate tokio_stream;
extern crate uritemplate;

pub mod adapters;
pub mod error;
pub mod private;
pub mod public;
pub mod request;

pub use adapters::{ASync, Sync};
pub use error::CBError;
pub use private::Private;
pub use public::Public;

pub const MAIN_URL: &str = "https://api.coinbase.com";

pub type DateTime = chrono::DateTime<chrono::Utc>;
