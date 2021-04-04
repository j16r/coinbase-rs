use std::fmt;

use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug, Error)]
pub enum CBError {
    #[error("http error {0}")]
    Http(#[from] super::hyper::Error),
    #[error(transparent)]
    Serde(#[from] super::serde_json::Error),
    #[error("coinbase: {0}")]
    Coinbase(Error),
    #[error("null")]
    Null,
}
