use std::future::Future;

use bytes::Bytes;
use http_body_util::{BodyExt as _, Full};
use hyper::{
    header::{HeaderValue, CONTENT_TYPE, USER_AGENT},
    Method, Request, StatusCode, Uri,
};
use hyper_rustls::{builderstates::WantsSchemes, HttpsConnector, HttpsConnectorBuilder};
use hyper_util::client::legacy::connect::HttpConnector;

pub(super) struct Client {
    inner: hyper_util::client::legacy::Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    user_agent: HeaderValue,
    content_type: HeaderValue,
}

impl Client {
    pub fn new() -> Client {
        let https = connection_builder().https_only().enable_http2().build();
        let user_agent = concat!("rust-", env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
        Self {
            inner: hyper_util::client::legacy::Builder::new(hyper_util::rt::TokioExecutor::new()).build(https),
            user_agent: HeaderValue::from_static(user_agent),
            content_type: HeaderValue::from_static("application/x-www-form-urlencoded"),
        }
    }

    pub fn request<T>(&self, uri: &Uri, body: &T) -> Request<Full<Bytes>>
    where
        T: serde::Serialize,
    {
        let mut req = Request::builder().uri(uri).method(Method::POST);
        let headers = req.headers_mut().unwrap();
        headers.insert(USER_AGENT, self.user_agent.clone());
        headers.insert(CONTENT_TYPE, self.content_type.clone());
        let body: Full<Bytes> = serde_urlencoded::to_string(body).unwrap().into();
        req.body(body).unwrap()
    }

    pub fn send<T>(&self, req: Request<Full<Bytes>>) -> impl Future<Output = crate::Result<T>> + Send + 'static
    where
        T: serde::de::DeserializeOwned,
    {
        let fut = self.inner.request(req);
        async {
            use bytes::Buf as _;

            let (parts, body) = fut.await?.into_parts();
            match parts.status {
                StatusCode::OK => {
                    let buf = body.collect().await?.to_bytes();
                    serde_json::from_reader(buf.reader()).map_err(crate::Error::JsonDeserialize)
                }
                _ => Err(crate::Error::StatusCode((parts, body))),
            }
        }
    }
}

#[cfg(feature = "native-certs")]
fn connection_builder() -> HttpsConnectorBuilder<WantsSchemes> {
    HttpsConnectorBuilder::new().with_native_roots().expect("HttpsConnectorBuilder::new().with_native_roots()")
}

#[cfg(all(not(feature = "native-certs"), feature = "webpki-roots"))]
fn connection_builder() -> HttpsConnectorBuilder<WantsSchemes> {
    HttpsConnectorBuilder::new().with_webpki_roots()
}
