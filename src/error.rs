/// Represents errors that can occur during finding credentials and fetching token.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    // common
    #[error("gcemeta client error: {0}")]
    Gcemeta(#[from] gcemeta::Error),
    // credentials
    #[error("api key format error: {0}")]
    ApiKeyFormat(hyper::http::uri::InvalidUri),
    #[error(
        "not found credentials source, please set the environment variable `RUST_LOG` to `google_authz=trace` for more details"
    )]
    CredentialsSource,
    #[error("read credentials file error: {0}")]
    CredentialsFile(std::io::Error),
    #[error("user or service account credentials format error: user={user}, service_account={service_account})")]
    CredentialsFormat { user: serde_json::Error, service_account: serde_json::Error },
    // authentication
    #[error("http client error: {0}")]
    Http(#[from] hyper_util::client::legacy::Error),
    #[error("response collection error: {0}")]
    Response(#[from] hyper::Error),
    #[error("response status code error: {0:?}")]
    StatusCode((hyper::http::response::Parts, hyper::body::Incoming)),
    #[error("response body deserialize error: {0}")]
    JsonDeserialize(serde_json::Error),
    #[error("token format error: type={0:?}, token={0:?}")]
    TokenFormat(String, String),
    #[cfg(not(feature = "tonic"))]
    #[error("uri schema error: {0:?}")]
    EnforceHttps(Option<String>),
}

/// Wrapper for the `Result` type.
pub type Result<T> = std::result::Result<T, Error>;
