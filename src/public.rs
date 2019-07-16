use super::error::CBError;
use crate::adapters::{Adapter, AdapterNew};

use hyper::client::HttpConnector;
use hyper::rt::{Future, Stream};
use hyper::{Body, Client, Request};
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
