use crate::adapters::{Adapter, AdapterNew};
use crate::public::Public;

use hmac::{Hmac, Mac};
use hyper::header::HeaderValue;
use hyper::{Body, Method, Request, Uri};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Private<Adapter> {
    _pub: Public<Adapter>,
    key: String,
    secret: String,
}

impl<A> Private<A> {
    pub fn new(uri: &str, key: &str, secret: &str) -> Self
    where
        A: AdapterNew,
    {
        Self {
            _pub: Public::new(uri),
            key: key.to_string(),
            secret: secret.to_string(),
        }
    }

    pub fn accounts(&self) -> A::Result
    where
        A: Adapter<Vec<Account>> + 'static,
    {
        self.call_get("/accounts")
    }

    fn call_get<U>(&self, uri: &str) -> A::Result
    where
        A: Adapter<U> + 'static,
        U: Send + 'static,
        for<'de> U: serde::Deserialize<'de>,
    {
        self.call(Method::GET, uri, "")
    }

    fn call<U>(&self, method: Method, uri: &str, body_str: &str) -> A::Result
    where
        A: Adapter<U> + 'static,
        U: Send + 'static,
        for<'de> U: serde::Deserialize<'de>,
    {
        self._pub
            .call(self.request(method, uri, body_str.to_string()))
    }

    fn request(&self, method: Method, _uri: &str, body_str: String) -> Request<Body> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("leap-second")
            .as_secs();

        let uri: Uri = (self._pub.uri.to_string() + _uri).parse().unwrap();

        let mut req = Request::builder();
        req.method(&method);
        req.uri(&uri);

        let sign = Self::sign(&self.secret, timestamp, method, &uri.path(), &body_str);

        req.header("User-Agent", Public::<A>::USER_AGENT);
        req.header("Content-Type", "Application/JSON");

        //req.header("CB-ACCESS-TOKEN", HeaderValue::from_str(&self.key).unwrap());
        req.header(
            "CB-ACCESS-VERSION",
            HeaderValue::from_str("2016-02-18").unwrap(),
        );
        req.header("CB-ACCESS-KEY", HeaderValue::from_str(&self.key).unwrap());
        req.header("CB-ACCESS-SIGN", HeaderValue::from_str(&sign).unwrap());
        req.header(
            "CB-ACCESS-TIMESTAMP",
            HeaderValue::from_str(&timestamp.to_string()).unwrap(),
        );

        req.body(body_str.into()).unwrap()
    }

    pub fn sign(
        secret: &str,
        timestamp: u64,
        method: Method,
        path: &str,
        body_str: &str,
    ) -> String {
        let mut mac: Hmac<sha2::Sha256> =
            Hmac::new_varkey(&secret.as_bytes()).expect("Hmac::new(secret)");
        let input = timestamp.to_string() + method.as_str() + path + body_str;
        dbg!(&input);
        mac.input(input.as_bytes());
        format!("{:x}", &mac.result().code())
    }
}

#[derive(Deserialize, Debug)]
pub struct Account {
    pub id: String,
    pub currency: String,
    pub balance: f64,
    pub available: f64,
    pub hold: f64,
    pub profile_id: String,
}

#[derive(Deserialize, Debug)]
pub struct AccountHistory {
    pub id: String,
}
