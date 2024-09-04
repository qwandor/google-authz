use std::{env, error::Error};

use bytes::Bytes;
use google_authz::{GoogleAuthz, ServiceError};
use http_body_util::{BodyExt as _, Either, Empty, Full};
use hyper::{body::Incoming, Request};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::client::legacy::connect::HttpConnector;
use tower_service::Service as _;

type Body = Either<Empty<Bytes>, Full<Bytes>>;

struct Client {
    inner: GoogleAuthz<hyper_util::client::legacy::Client<HttpsConnector<HttpConnector>, Body>>,
}

impl Client {
    async fn try_new() -> Result<Self, Box<dyn Error>> {
        let https = HttpsConnectorBuilder::new().with_native_roots()?.https_only().enable_http2().build();
        let client = hyper_util::client::legacy::Builder::new(hyper_util::rt::TokioExecutor::new()).build(https);
        let inner = GoogleAuthz::new(client).init().await?;
        Ok(Self { inner })
    }

    async fn request(
        &mut self,
        req: hyper::Request<Body>,
    ) -> Result<hyper::Response<Incoming>, ServiceError<hyper_util::client::legacy::Error>> {
        futures_util::future::poll_fn(|cx| self.inner.poll_ready(cx)).await?;
        self.inner.call(req).await
    }
}

// https://cloud.google.com/pubsub/docs/reference/rest/v1/projects.topics/list
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let project = env::args().nth(1).expect("cargo run --bin hyper -- <GCP_PROJECT_ID>");

    let mut client = Client::try_new().await?;
    let mut req = Request::new(Either::Left(Empty::new()));
    *req.uri_mut() =
        format!("https://pubsub.googleapis.com/v1/projects/{project}/topics?alt=json&prettyPrint=true").parse()?;
    let resp = client.request(req).await?;

    let (parts, body) = resp.into_parts();
    println!("response parts = {:?}", parts);

    let body = String::from_utf8(body.collect().await?.to_bytes().into())?;
    println!("resposne body = `{}`", body);

    Ok(())
}
