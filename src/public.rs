use std::collections::HashMap;
// use std::future::Future;
use std::thread;
use std::time::Duration;

use async_stream::try_stream;
use bigdecimal::BigDecimal;
use hyper::{Body, Client, client::HttpConnector, Request, Uri};
use hyper_tls::HttpsConnector;
use futures::stream::Stream;
use futures::Future;

use crate::adapters::{Adapter, AdapterNew};
use crate::DateTime;
use super::error::CBError;

pub struct Public<Adapter> {
    pub(crate) uri: String,
    pub(crate) adapter: Adapter,
    client: Client<HttpsConnector<HttpConnector>>,
}

impl<A> Public<A> {
    pub(crate) const USER_AGENT: &'static str = concat!("coinbase-rs/", env!("CARGO_PKG_VERSION"));

    pub fn new(uri: &str) -> Self
    where
        A: AdapterNew,
    {
        let https = HttpsConnector::new();
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .build::<_, Body>(https);
        let uri = uri.to_string();

        Self {
            uri,
            client,
            adapter: A::new().expect("Failed to initialize adapter"),
        }
    }

    pub(crate) fn call_future<U>(
        &self,
        request: Request<Body>,
    ) -> impl Future<Output = Result<Response<U>, CBError>>
    where
        U: serde::de::DeserializeOwned,
    {
        thread::sleep(Duration::from_millis(350));

        let request_future = self.client.request(request);

        async move {
            let response = request_future.await?;
            let body = hyper::body::to_bytes(response.into_body()).await?;

            match serde_json::from_slice::<Response<U>>(&body) {
                Ok(body) => Ok(body),
                Err(e) => match serde_json::from_slice(&body) {
                    Ok(coinbase_err) => Err(CBError::Coinbase(coinbase_err)),
                    Err(_) => Err(CBError::Serde(e)),
                },
            }
        }
    }

    pub(crate) fn call<U>(&self, request: Request<Body>) -> A::Result
    where
        A: Adapter<U> + 'static,
        U: Send + 'static,
        U: serde::de::DeserializeOwned,
    {
        self.adapter.process(self.call_future(request))
    }

    pub(crate) fn fetch_stream<'a, U>(&'a self, uri: String) -> impl Stream<Item = Result<U, CBError>> + 'a
    where
        A: Adapter<U> + 'static,
        U: Send + 'static,
        U: serde::de::DeserializeOwned,
        U: std::marker::Unpin,
    {
        try_stream! {
            let initial_request = self.request(&uri);
            let result = self.call_future(initial_request).await?;
            yield result.data;

            while let(Some(ref next_uri)) = result.pagination.next_uri {
                let request = self.request(next_uri);
                let result = self.call_future(request).await?;
                yield result.data;
            }
        }
    }

    fn get_pub<U>(&self, uri: &str) -> A::Result
    where
        A: Adapter<U> + 'static,
        U: Send + 'static,
        U: serde::de::DeserializeOwned,
    {
        self.call(self.request(uri))
    }

    fn request(&self, uri: &str) -> Request<Body> {
        let uri: Uri = (self.uri.to_string() + uri).parse().unwrap();

        Request::get(uri)
            .header("User-Agent", Self::USER_AGENT)
            .body(Body::empty())
            .unwrap()
    }

    ///
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

    ///
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

    ///
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

    ///
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

    ///
    /// **Get spot price**
    ///
    /// Get the current market price for a currency pair. This is usually somewhere in between the
    /// buy and sell price.
    ///
    /// https://developers.coinbase.com/api/v2#get-spot-price
    ///
    pub fn spot_price(&self, currency_pair: &str, _date: Option<chrono::NaiveDate>) -> A::Result
    where
        A: Adapter<CurrencyPrice> + 'static,
    {
        self.get_pub(&format!("/currency_pair/{}/spot", currency_pair))
    }

    ///
    /// **Get current time**
    ///
    /// Get the API server time.
    ///
    /// https://developers.coinbase.com/api/v2#time
    ///
    pub fn current_time(&self) -> A::Result
    where
        A: Adapter<DateTime> + 'static,
    {
        self.get_pub("/current_time")
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Response<U> {
    pub pagination: Pagination,
    pub data: U,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Order {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

#[derive(Deserialize, Serialize, Debug)]
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

#[derive(Deserialize, Debug)]
struct CurrentTime {
    iso: DateTime,
}

#[cfg(test)]
mod test {
    use bigdecimal::FromPrimitive;

    use super::*;

    #[test]
    fn test_currencies_deserialize() {
        let input = r#"
    [
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
        let currencies: Vec<Currency> = serde_json::from_slice(input.as_bytes()).unwrap();
        assert_eq!(currencies.len(), 4);
    }

    #[test]
    fn test_exchange_rates_deserialize() {
        let input = r#"
    {
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
        let exchange_rates: ExchangeRates = serde_json::from_slice(input.as_bytes()).unwrap();
        assert_eq!(exchange_rates.currency, "BTC");
        assert_eq!(exchange_rates.rates.len(), 11);
    }

    #[test]
    fn test_currency_price_deserialize() {
        let input = r#"
    {
    "amount": "1010.25",
    "currency": "USD"
    }"#;
        let currency_price: CurrencyPrice = serde_json::from_slice(input.as_bytes()).unwrap();
        assert_eq!(currency_price.amount, BigDecimal::from_f32(1010.25).unwrap());
        assert_eq!(currency_price.currency, "USD");
    }

    #[test]
    fn test_current_time_deserialize() {
        let input = r#"
    {
    "iso": "2015-06-23T18:02:51Z",
    "epoch": 1435082571
    }"#;
        let time: crate::DateTime = serde_json::from_slice(input.as_bytes())
            .map(|c: CurrentTime| c.iso)
            .unwrap();
        assert_eq!(1435082571, time.timestamp());
    }
}
