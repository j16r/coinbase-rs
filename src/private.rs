use crate::adapters::{Adapter, AdapterNew};
use crate::public::Public;
use crate::DateTime;

use bigdecimal::BigDecimal;
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

    ///
    /// **List accounts**
    ///
    /// Lists current user’s accounts to which the authentication method has access to.
    ///
    /// https://developers.coinbase.com/api/v2#list-accounts
    ///
    pub fn accounts(&self) -> A::Result
    where
        A: Adapter<Vec<Account>> + 'static,
    {
        self.call_get("/accounts")
    }

    ///
    /// **List transactions**
    ///
    /// Lists account’s transactions.
    ///
    /// https://developers.coinbase.com/api/v2#list-transactions
    ///
    pub fn transactions(&self, account_id: &Uuid) -> A::Result
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
    // id appears to be either a UUID or a token name e.g: "LINK"
    pub id: String,

    pub r#type: String,

    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,

    pub resource: String,
    pub resource_path: String,

    pub name: String,
    pub primary: bool,

    pub currency: Currency,

    pub balance: Balance,

    pub allow_deposits: bool,
    pub allow_withdrawals: bool,
}

#[derive(Deserialize, Debug)]
pub struct Balance {
    pub amount: BigDecimal,
    pub currency: String,
}

#[derive(Deserialize, Debug)]
pub struct Transaction {
    pub id: Uuid,

    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,

    pub r#type: String,
    pub resource: String,
    pub resource_path: String,
    pub status: String,
    pub amount: Balance,
    pub native_amount: Balance,
    pub instant_exchange: bool,
    pub network: Option<Network>,
    pub from: Option<From>,
    pub details: TransactionDetails,
}

#[derive(Deserialize, Debug)]
pub struct Network {
    pub status: String,
}

#[derive(Deserialize, Debug)]
pub struct From {
    pub id: Option<Uuid>,
    pub resource: String,
    pub resource_path: Option<String>,
    pub currency: String,
}

#[derive(Deserialize, Debug)]
pub struct TransactionDetails {
    pub title: String,
    pub subtitle: String,
}

#[derive(Deserialize, Debug)]
pub struct Currency {
    pub code: String,
    pub name: String,
    pub color: String,
    pub sort_index: usize,
    pub exponent: usize,
    pub r#type: String,
    pub address_regex: Option<String>,
    pub asset_id: Option<Uuid>,
    pub destination_tag_name: Option<String>,
    pub destination_tag_regex: Option<String>,
}

#[test]
fn test_account_deserialize() {
    let input = r##"[
{
  "id": "f1bb8f61-7f5d-4f04-9552-bcbafdf856b7",
  "type": "wallet",
  "created_at": "2019-07-12T03:27:07Z",
  "updated_at": "2019-07-12T14:07:57Z",
  "resource": "account",
  "resource_path": "/v2/accounts/f1bb8f61-7f5d-4f04-9552-bcbafdf856b7",
  "name": "EOS Wallet",
  "primary": true,
  "currency": {
    "code": "EOS",
    "name": "EOS",
    "color": "#000000",
    "sort_index": 128,
    "exponent": 4,
    "type": "crypto",
    "address_regex": "(^[a-z1-5.]{1,11}[a-z1-5]$)|(^[a-z1-5.]{12}[a-j1-5]$)",
    "asset_id": "cc2ddaa5-5a03-4cbf-93ef-e4df102d4311",
    "destination_tag_name": "EOS Memo",
    "destination_tag_regex": "^.{1,100}$"
  },
  "balance": {
    "amount": "9.1238",
    "currency": "EOS"
  },
  "allow_deposits": true,
  "allow_withdrawals": true
}
]"##;

    let accounts: Vec<Account> = serde_json::from_slice(input.as_bytes()).unwrap();
    assert_eq!(accounts.len(), 1);
}

#[test]
fn test_transactions_deserialize() {
    let input = r#"[
{
  "id": "9dd482e4-d8ce-46f7-a261-281843bd2855",
  "type": "send",
  "status": "completed",
  "amount": {
    "amount": "-0.00100000",
    "currency": "BTC"
  },
  "native_amount": {
    "amount": "-0.01",
    "currency": "USD"
  },
  "description": null,
  "created_at": "2015-03-11T13:13:35-07:00",
  "updated_at": "2015-03-26T15:55:43-07:00",
  "resource": "transaction",
  "resource_path": "/v2/accounts/af6fd33a-e20c-494a-b3f6-f91d204af4b7/transactions/9dd482e4-d8ce-46f7-a261-281843bd2855",
  "network": {
    "status": "off_blockchain",
    "name": "bitcoin"
  },
  "to": {
    "id": "2dbc3cfb-ed1e-4c10-aedb-aeb1693e01e7",
    "resource": "user",
    "resource_path": "/v2/users/2dbc3cfb-ed1e-4c10-aedb-aeb1693e01e7"
  },
  "instant_exchange": false,
  "details": {
    "title": "Sent bitcoin",
    "subtitle": "to User 2"
  }
},
{
  "id": "c1c413d1-acf8-4fcb-a8ed-4e2e4820c6f0",
  "type": "buy",
  "status": "pending",
  "amount": {
    "amount": "1.00000000",
    "currency": "BTC"
  },
  "native_amount": {
    "amount": "10.00",
    "currency": "USD"
  },
  "description": null,
  "created_at": "2015-03-26T13:42:00-07:00",
  "updated_at": "2015-03-26T15:55:45-07:00",
  "resource": "transaction",
  "resource_path": "/v2/accounts/af6fd33a-e20c-494a-b3f6-f91d204af4b7/transactions/c1c413d1-acf8-4fcb-a8ed-4e2e4820c6f0",
  "buy": {
    "id": "ae7df6e7-fef1-441d-a6f3-e4661ca6f39a",
    "resource": "buy",
    "resource_path": "/v2/accounts/af6fd33a-e20c-494a-b3f6-f91d204af4b7/buys/ae7df6e7-fef1-441d-a6f3-e4661ca6f39a"
  },
  "instant_exchange": false,
  "details": {
    "title": "Bought bitcoin",
    "subtitle": "using Capital One Bank"
  }
}
]"#;
    let transactions: Vec<Transaction> = serde_json::from_slice(input.as_bytes()).unwrap();
    assert_eq!(transactions.len(), 2);
}
