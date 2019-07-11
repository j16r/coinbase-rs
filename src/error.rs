use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug, Fail)]
pub enum CBError {
    #[fail(display = "http: {}", _0)]
    Http(#[cause] super::hyper::Error),
    #[fail(display = "serde: {}\n    {}", error, data)]
    Serde {
        #[cause]
        error: super::serde_json::Error,
        data: String,
    },
    #[fail(display = "coinbase: {}", _0)]
    Coinbase(Error),
    #[fail(display = "null")]
    Null,
}
