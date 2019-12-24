use hyper_tls::HttpsConnector;
use hyper::client::{Client, HttpConnector, ResponseFuture};
use hyper::Body;

use http::method::Method;
use http::uri::Uri;
use http::header::{HeaderMap, HeaderValue};
use http::request::Builder;

use crate::B2Error;
use std::future::Future;

use serde::Serialize;

type HyperClient = Client<HttpsConnector<HttpConnector>, Body>;

/// A client for interacting with the b2 api.
#[derive(Clone, Debug)]
pub struct B2Client {
    inner: HyperClient,
}

impl B2Client {
    /// Creates a new client with the default hyper backend.
    pub fn new() -> Self {
        Self::with_client(
            Client::builder().build(HttpsConnector::new())
        )
    }
    /// Creates a new client with the provided hyper backend.
    pub fn with_client(client: HyperClient) -> Self {
        Self { inner: client }
    }
    /// This function starts the provided api call. As this returns a future, you will
    /// need to await it to obtain the resulting value.
    pub fn send<Api: ApiCall>(&mut self, api: Api) -> Api::Future {
        let url = match api.url() {
            Ok(url) => url,
            Err(err) => return api.error(err),
        };
        let mut builder = Builder::new()
            .method(Api::METHOD)
            .uri(url);
        if let Some(headers_mut) = builder.headers_mut() {
            match api.headers() {
                Ok(headers) => {
                    *headers_mut = headers;
                },
                Err(err) => return api.error(err),
            }
        }
        match api.body().and_then(|body| builder.body(body).map_err(B2Error::from)) {
            Ok(request) => api.finalize(self.inner.request(request)),
            Err(err) => api.error(err),
        }
    }
}

/// An api call that the [`B2Client`] can execute.
///
/// This trait is implemented by every api call, so you can see a list of api calls in
/// [the implementors section](#implementors).
///
/// In order to use new b2 api calls before they are officially supported in this
/// library, it is possible to manually implement this trait for your own api call type.
///
/// [`B2Client`]: struct.B2Client.html
pub trait ApiCall {
    /// The type of future used by this api call.
    type Future: Future;
    /// The http method used by the api call.
    const METHOD: Method;
    /// The url for this api call.
    fn url(&self) -> Result<Uri, B2Error>;
    /// Any headers needed by the request.
    fn headers(&self) -> Result<HeaderMap<HeaderValue>, B2Error>;
    /// The body of the request.
    fn body(&self) -> Result<Body, B2Error>;
    /// Wrap the `ResponseFuture` in a future that handles the response.
    fn finalize(&self, fut: ResponseFuture) -> Self::Future;
    /// Create a future that immediately fails with the supplied error.
    fn error(&self, err: B2Error) -> Self::Future;
}

#[inline]
pub(crate) fn serde_body<T: Serialize + ?Sized>(body: &T) -> Result<Body, B2Error> {
    Ok(Body::from(serde_json::to_vec(body)?))
}
