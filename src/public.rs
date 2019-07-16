use super::error::CBError;
use crate::adapters::{Adapter, AdapterNew};

use bigdecimal::BigDecimal;
use hyper::client::HttpConnector;
use hyper::rt::{Future, Stream};
use hyper::{Body, Client, Request, Uri};
use hyper_tls::HttpsConnector;

pub struct Public<Adapter> {
    pub(crate) uri: String,
    pub(crate) adapter: Adapter,
    client: Client<HttpsConnector<HttpConnector>>,
}

impl<A> Public<A> {
    pub(crate) const USER_AGENT: &'static str = concat!("coinbase-rs/", env!("CARGO_PKG_VERSION"));

    pub fn new_with_keep_alive(uri: &str, keep_alive: bool) -> Self
    where
        A: AdapterNew,
    {
        let https = HttpsConnector::new(4).unwrap();
        let client = Client::builder()
            .keep_alive(keep_alive)
            .build::<_, Body>(https);
        let uri = uri.to_string();

        Self {
            uri,
            client,
            adapter: A::new().expect("Failed to initialize adapter"),
        }
    }

    pub fn new(uri: &str) -> Self
    where
        A: AdapterNew,
    {
        Self::new_with_keep_alive(uri, true)
    }

    pub(crate) fn call_future<U>(
        &self,
        request: Request<Body>,
    ) -> impl Future<Item = U, Error = CBError>
    where
        for<'de> U: serde::Deserialize<'de>,
    {
        self.client
            .request(request)
            .map_err(CBError::Http)
            .and_then(|res| res.into_body().concat2().map_err(CBError::Http))
            .and_then(|body| {
                let res: serde_json::Value = serde_json::from_slice(&body).map_err(|e| {
                    serde_json::from_slice(&body)
                        .map(CBError::Coinbase)
                        .unwrap_or_else(|_| {
                            let data = String::from_utf8(body.to_vec()).unwrap();
                            CBError::Serde { error: e, data }
                        })
                })?;
                let data = serde_json::from_slice(res["data"].to_string().as_bytes())
                    .expect("parsing Response.data");
                Ok(data)
            })
    }

    pub(crate) fn call<U>(&self, request: Request<Body>) -> A::Result
    where
        A: Adapter<U> + 'static,
        U: Send + 'static,
        for<'de> U: serde::Deserialize<'de>,
    {
        self.adapter.process(self.call_future(request))
    }

    fn get_pub<U>(&self, uri: &str) -> A::Result
    where
        A: Adapter<U> + 'static,
        U: Send + 'static,
        for<'de> U: serde::Deserialize<'de>,
    {
        self.call(self.request(uri))
    }

    fn request(&self, uri: &str) -> Request<Body> {
        let uri: Uri = (self.uri.to_string() + uri).parse().unwrap();

        let mut req = Request::get(uri);
        req.header("User-Agent", Self::USER_AGENT);
        req.body(Body::empty()).unwrap()
    }

    /// **Get currencies**
    ///
    /// List known currencies. Currency codes will conform to the ISO 4217 standard where possible.
    /// Currencies which have or had no representation in ISO 4217 may use a custom code (e.g.
    /// BTC).
    ///
    /// # Account Fields
    /// | Field    | Description             |
    /// | -------- | ----------------------- |
    /// | id       | ISO 4217 currency code  |
    /// | name     | Name of currency        |
    /// | min_size |                         |
    ///
    pub fn currencies(&self) -> A::Result
    where
        A: Adapter<Vec<Currency>> + 'static,
    {
        self.get_pub("/currencies")
    }
}

type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Deserialize, Debug)]
pub struct Response {
    pub pagination: Pagination,
    pub data: serde_json::Value,
}

#[derive(Deserialize, Debug)]
pub struct Pagination {
    pub ending_before: Option<DateTime>,
    pub starting_after: Option<DateTime>,
    pub previous_ending_before: Option<DateTime>,
    pub next_starting_after: Option<DateTime>,
    pub limit: usize,
    pub order: String,
    pub previous_uri: String,
    pub next_uri: String,
}

#[derive(Deserialize, Debug)]
pub struct Currency {
    pub id: String,
    pub name: String,
    pub min_size: BigDecimal,
}

#[test]
fn test_currencies_deserialize() {
    let input = r#"[
{
  "id": "AED",
  "name": "United Arab Emirates Dirham",
  "min_size": "0.01000000"
},
{
  "id": "AFN",
  "name": "Afghan Afghani",
  "min_size": "0.01000000"
},
{
  "id": "ALL",
  "name": "Albanian Lek",
  "min_size": "0.01000000"
},
{
  "id": "AMD",
  "name": "Armenian Dram",
  "min_size": "0.01000000"
}
]"#;
    let accounts: Vec<Currency> = serde_json::from_slice(input.as_bytes()).unwrap();
}
