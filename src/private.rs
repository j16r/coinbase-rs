use crate::adapters::{Adapter, AdapterNew};
use crate::public::Public;

use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use hyper::header::HeaderValue;
use hyper::{Body, Method, Request, Uri};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

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
        self.call_get("/accounts?limit=100")
    }

    pub fn list_transactions(&self, account_id: &Uuid) -> A::Result
    where
        A: Adapter<Vec<Transaction>> + 'static,
    {
        self.call_get(&format!("/accounts/{}/transactions", account_id))
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

        let sign = Self::sign(
            &self.secret,
            timestamp,
            method,
            &uri.path_and_query().unwrap().as_str(),
            &body_str,
        );

        req.header("User-Agent", Public::<A>::USER_AGENT);
        req.header("Content-Type", "Application/JSON");

        req.header("CB-VERSION", HeaderValue::from_str("2019-04-03").unwrap());
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
        let mut mac: Hmac<Sha256> =
            Hmac::new_varkey(&secret.as_bytes()).expect("Hmac::new(secret)");
        let input = timestamp.to_string() + method.as_str() + path + body_str;
        mac.input(input.as_bytes());
        format!("{:x}", &mac.result().code())
    }
}

#[derive(Deserialize, Debug)]
pub struct Account {
    pub id: String,

    pub allow_deposits: bool,
    pub allow_withdrawals: bool,
    pub balance: Balance,
    pub created_at: Option<DateTime<Utc>>,
    pub currency: String,

    pub name: String,
    pub native_balance: Balance,
    pub primary: bool,
    pub resource: String,
    pub resource_path: String,
    pub r#type: String,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Debug)]
pub struct Balance {
    pub amount: BigDecimal,
    pub currency: String,
}

#[derive(Deserialize, Debug)]
pub struct AccountHistory {
    pub id: String,
}

#[test]
fn test_account_deserialize() {
    let input = r#"[
{
    "allow_deposits": true,
    "allow_withdrawals": true,
    "balance": {
        "amount": "2.0964",
        "currency": "EOS"
    },
    "created_at": "2019-07-12T03:27:07Z",
    "currency": "EOS",
    "id": "b95a5b5b-9ed6-5486-85bc-d1a4052f2023",
    "name": "EOS Wallet",
    "native_balance": {
        "amount": "9.96",
        "currency": "USD"
    },
    "primary": false,
    "resource": "account",
    "resource_path": "/v2/accounts/b95a5b5b-9ed6-5486-85bc-d1a4052f2023",
    "type": "wallet",
    "updated_at": "2019-07-12T14:07:57Z"
}
]"#;
    let accounts: Vec<Account> = serde_json::from_slice(input.as_bytes()).unwrap();
}
