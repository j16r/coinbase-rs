use super::error::CBError;
use crate::adapters::{Adapter, AdapterNew};
use std::collections::HashMap;

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
    /// https://developers.coinbase.com/api/v2#currencies
    ///
    pub fn currencies(&self) -> A::Result
    where
        A: Adapter<Vec<Currency>> + 'static,
    {
        self.get_pub("/currencies")
    }

    /// **Get exchange rates**
    ///
    /// Get current exchange rates. Default base currency is USD but it can be defined as any
    /// supported currency. Returned rates will define the exchange rate for one unit of the base
    /// currency.
    ///
    /// https://developers.coinbase.com/api/v2#exchange-rates
    ///
    pub fn exchange_rates(&self) -> A::Result
    where
        A: Adapter<ExchangeRates> + 'static,
    {
        self.get_pub("/exchange-rates")
    }

    /// **Get buy price**
    ///
    /// Get the total price to buy one bitcoin or ether.
    ///
    /// https://developers.coinbase.com/api/v2#get-buy-price
    ///
    pub fn buy_price(&self, currency_pair: &str) -> A::Result
    where
        A: Adapter<CurrencyPrice> + 'static,
    {
        self.get_pub(&format!("/currency_pair/{}/buy", currency_pair))
    }

    /// **Get sell price**
    ///
    /// Get the total price to sell one bitcoin or ether.
    ///
    /// https://developers.coinbase.com/api/v2#get-sell-price
    ///
    pub fn sell_price(&self, currency_pair: &str) -> A::Result
    where
        A: Adapter<CurrencyPrice> + 'static,
    {
        self.get_pub(&format!("/currency_pair/{}/sell", currency_pair))
    }

    /// **Get spot price**
    ///
    /// Get the current market price for a currency pair. This is usually somewhere in between the
    /// buy and sell price.
    ///
    /// https://developers.coinbase.com/api/v2#get-spot-price
    ///
    pub fn spot_price(&self, currency_pair: &str, date: Option<chrono::NaiveDate>) -> A::Result
    where
        A: Adapter<CurrencyPrice> + 'static,
    {
        self.get_pub(&format!("/currency_pair/{}/spot", currency_pair))
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

#[derive(Deserialize, Debug)]
pub struct ExchangeRates {
    pub currency: String,
    pub rates: HashMap<String, BigDecimal>,
}

#[derive(Deserialize, Debug)]
pub struct CurrencyPrice {
    pub amount: BigDecimal,
    pub currency: String,
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

#[test]
fn test_exchange_rates_deserialize() {
    let input = r#"{
  "currency": "BTC",
  "rates": {
    "AED": "36.73",
    "AFN": "589.50",
    "ALL": "1258.82",
    "AMD": "4769.49",
    "ANG": "17.88",
    "AOA": "1102.76",
    "ARS": "90.37",
    "AUD": "12.93",
    "AWG": "17.93",
    "AZN": "10.48",
    "BAM": "17.38"
  }
}"#;
    let accounts: ExchangeRates = serde_json::from_slice(input.as_bytes()).unwrap();
}

#[test]
fn test_currency_price_deserialize() {
    let input = r#"{
  "amount": "1010.25",
  "currency": "USD"
}"#;
    let accounts: CurrencyPrice = serde_json::from_slice(input.as_bytes()).unwrap();
}
