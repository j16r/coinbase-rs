use std::collections::HashMap;
use std::result;
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use http::{request, Method, Request, Uri, Version};
use hyper::Body;
use sha2::Sha256;

#[derive(Debug)]
pub struct Error {}

pub type Result<T> = result::Result<T, Error>;

type HmacSha256 = Hmac<Sha256>;

const USER_AGENT: &str = concat!("coinbase-rs/", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Debug, Default)]
pub struct Parts {
    /// The request's method
    pub method: Method,

    /// The request's URI
    pub uri: Uri,

    /// The request's version
    pub version: Version,

    /// The request's headers
    pub headers: HashMap<String, String>,
}

#[derive(Clone, Debug, Default)]
pub struct Builder {
    auth: Option<(String, String)>,
    parts: Parts,
    body: Vec<u8>,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            auth: None,
            parts: Parts {
                method: Method::GET,
                uri: "/".parse().unwrap(),
                version: Version::default(),
                headers: HashMap::new(),
            },
            body: Vec::new(),
        }
    }

    pub fn new_with_auth(key: &str, secret: &str) -> Builder {
        Builder {
            auth: Some((key.to_string(), secret.to_string())),
            parts: Parts {
                method: Method::GET,
                uri: "/".parse().unwrap(),
                version: Version::default(),
                headers: HashMap::new(),
            },
            body: Vec::new(),
        }
    }

    pub fn method(self, method: Method) -> Builder {
        let mut _self = self;
        _self.parts.method = method;
        _self
    }

    pub fn uri(self, uri: Uri) -> Builder {
        let mut _self = self;
        _self.parts.uri = uri;
        _self
    }

    pub fn version(self, version: Version) -> Builder {
        let mut _self = self;
        _self.parts.version = version;
        _self
    }

    pub fn header(self, key: &str, value: &str) -> Builder {
        let mut _self = self;
        _self.parts.headers.insert(key.into(), value.into());
        _self
    }

    pub fn body(self, body: &Vec<u8>) -> Builder {
        let mut _self = self;
        _self.body = body.clone();
        _self
    }

    pub fn build(self) -> Request<Body> {
        let _self = if let Some((ref key, ref secret)) = self.auth {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("leap-second")
                .as_secs();

            let sign = Self::sign(
                secret,
                timestamp,
                &self.parts.method,
                self.parts.uri.path_and_query().unwrap().as_str(),
                &self.body,
            );

            self.clone()
                .header("User-Agent", USER_AGENT)
                .header("Content-Type", "Application/JSON")
                .header("CB-VERSION", "2021-01-01")
                .header("CB-ACCESS-KEY", key)
                .header("CB-ACCESS-SIGN", &sign)
                .header("CB-ACCESS-TIMESTAMP", &timestamp.to_string())
        } else {
            self
        };

        let mut builder = request::Builder::new()
            .method(_self.parts.method)
            .uri(_self.parts.uri);
        for (key, value) in _self.parts.headers {
            builder = builder.header(&key, &value);
        }
        builder.body(_self.body.into()).unwrap()
    }

    fn sign(secret: &str, timestamp: u64, method: &Method, path: &str, body: &Vec<u8>) -> String {
        let mut mac: Hmac<Sha256> =
            HmacSha256::new_varkey(&secret.as_bytes()).expect("Hmac::new(secret)");
        let input = timestamp.to_string() + method.as_str() + path;
        mac.input(input.as_bytes());
        mac.input(body);
        format!("{:x}", &mac.result().code())
    }
}
