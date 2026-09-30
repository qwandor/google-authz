use std::env;

use google_api_proto::google::pubsub::v1::{publisher_client::PublisherClient, ListTopicsRequest};
use google_authz::GoogleAuthz;
use tonic::{
    transport::{Channel, ClientTlsConfig},
    Request,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let project = env::args().nth(1).expect("cargo run --bin tonic -- <GCP_PROJECT_ID>");
    let channel = Channel::from_static("https://pubsub.googleapis.com")
        .tls_config(ClientTlsConfig::new().with_native_roots())?
        .connect()
        .await?;
    let channel = GoogleAuthz::new(channel).init().await?;

    let mut client = PublisherClient::new(channel);
    let response = client
        .list_topics(Request::new(ListTopicsRequest {
            project: format!("projects/{}", project),
            page_size: 10,
            ..Default::default()
        }))
        .await?;
    println!("response = {:#?}", response);

    Ok(())
}
