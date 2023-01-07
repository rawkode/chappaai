use axum::response::IntoResponse;
use hyper::StatusCode;
use kube::runtime::reflector::Store;
use thiserror::Error;
use tracing::subscriber::SetGlobalDefaultError;
use tracing_subscriber::filter::ParseError;

pub mod kubernetes;
pub mod oauth_api;
use crate::oauth_api::OAuthApi;
pub mod oauth_connection;
use crate::oauth_connection::OAuthConnection;

const RESOURCE_NAMESPACE: &str = "chappaai.dev";
const RESOURCE_VERSION: &str = "v1";

fn api_version() -> String {
    format!("{}/{}", RESOURCE_NAMESPACE, RESOURCE_VERSION)
}

pub struct ApiData {
    pub client: kube::Client,
    pub oauth_apis: Store<OAuthApi>,
    pub oauth_connections: Store<OAuthConnection>,
}

pub struct ApplicationState {
    pub client: kube::Client,
    pub oauth_apis: Store<OAuthApi>,
    pub oauth_connections: Store<OAuthConnection>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Generic Error: {0}")]
    GenericError(String),

    #[error("Kube Api Error: {0}")]
    KubeError(#[from] kube::Error),

    #[error("SerializationError: {0}")]
    SerializationError(#[source] serde_json::Error),

    #[error("Hyper Error: {0}")]
    HyperError(#[from] hyper::Error),

    #[error("SetGlobalDefaultError: {0}")]
    SetGlobalDefaultError(#[from] SetGlobalDefaultError),

    #[error("ParseError: {0}")]
    ParseError(#[from] ParseError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
