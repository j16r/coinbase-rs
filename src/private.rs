use bigdecimal::BigDecimal;
use futures::stream::Stream;
use hyper::Uri;
use uritemplate::UriTemplate;
use uuid::Uuid;

use crate::DateTime;
use crate::adapters::{Adapter, AdapterNew};
use crate::public::Public;
use crate::request;
use super::error::CBError;

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
        self.fetch("/v2/accounts")
    }

    pub fn accounts_stream<'a>(&'a self) -> impl Stream<Item = Result<Vec<Account>, CBError>> + 'a
    where
        A: Adapter<Vec<Account>> + 'static,
    {
        let limit = 100;
        let uri = UriTemplate::new("/v2/accounts{?query*}")
            .set("query", &[("limit", limit.to_string().as_ref())])
            .build();
        let request = self.request(&uri);
        self._pub.fetch_stream(request)
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
        let uri = UriTemplate::new("/v2/accounts/{account}/transactions")
            .set("account", account_id.to_string())
            .build();
        self.fetch(&uri)
    }

    ///
    /// **List transactions**
    ///
    /// Lists account’s transactions.
    ///
    /// https://developers.coinbase.com/api/v2#list-transactions
    ///
    pub fn transactions_stream<'a>(&'a self, account_id: &Uuid) -> impl Stream<Item = Result<Vec<Transaction>, CBError>> + 'a
    where
        A: Adapter<Vec<Transaction>> + 'static,
    {
        let limit = 100;
        let uri = UriTemplate::new("/v2/accounts/{account}/transactions{?query*}")
            .set("account", account_id.to_string())
            .set("query", &[("limit", limit.to_string().as_ref())])
            .build();
        let request = self.request(&uri);
        self._pub.fetch_stream(request)
    }

    fn fetch<U>(&self, uri: &str) -> A::Result
    where
        A: Adapter<U> + 'static,
        U: Send + 'static,
        U: serde::de::DeserializeOwned,
    {
        let request = self.request(uri);
        self._pub.call(request)
    }

    fn request(&self, _uri: &str) -> request::Builder {
        let uri: Uri = (self._pub.uri.to_string() + _uri).parse().unwrap();
        request::Builder::new_with_auth(&self.key, &self.secret)
            .uri(uri)
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

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub enum Order {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

#[derive(Deserialize, Debug)]
pub struct Pagination {
    pub ending_before: Option<DateTime>,
    pub starting_after: Option<DateTime>,
    pub previous_ending_before: Option<String>,
    pub next_starting_after: Option<String>,
    pub limit: usize,
    pub order: Order,
    pub previous_uri: Option<String>,
    pub next_uri: Option<String>,
}

#[test]
fn test_pagination_deserialize() {
    let input = r##"
{
    "ending_before": null,
    "starting_after": null,
    "previous_ending_before": null,
    "next_starting_after": "d16ec1ba-b3f7-5d6a-a9c8-817930030324",
    "limit": 25,
    "order": "desc",
    "previous_uri": null,
    "next_uri": "/v2/accounts?starting_after=d16ec1ba-b3f7-5d6a-a9c8-817930030324"
}"##;
    let pagination: Pagination = serde_json::from_slice(input.as_bytes()).unwrap();
    assert_eq!(25, pagination.limit);
    assert_eq!(Order::Descending, pagination.order);
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
