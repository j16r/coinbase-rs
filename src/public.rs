use super::error::CBError;
use crate::adapters::{Adapter, AdapterNew};

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

    // This function is contructor which can control keep_alive flag of the connection.
    // Created for tests to exit tokio::run
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

    fn request(&self, uri: &str) -> Request<Body> {
        let uri: Uri = (self.uri.to_string() + uri).parse().unwrap();

        let mut req = Request::get(uri);
        req.header("User-Agent", Self::USER_AGENT);
        req.body(Body::empty()).unwrap()
    }

    pub(crate) fn call_future<U>(
        &self,
        request: Request<Body>,
    ) -> impl Future<Item = U, Error = CBError>
    where
        for<'de> U: serde::Deserialize<'de>,
    {
        dbg!(&request);

        self.client
            .request(request)
            .map_err(CBError::Http)
            .and_then(|res| res.into_body().concat2().map_err(CBError::Http))
            .and_then(|body| {
                dbg!(&body);
                let res = serde_json::from_slice(&body).map_err(|e| {
                    serde_json::from_slice(&body)
                        .map(CBError::Coinbase)
                        .unwrap_or_else(|_| {
                            let data = String::from_utf8(body.to_vec()).unwrap();
                            CBError::Serde { error: e, data }
                        })
                })?;
                Ok(res)
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
