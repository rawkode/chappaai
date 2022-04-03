use kube::runtime::reflector::Store;
use thiserror::Error;

pub mod kubernetes;
pub mod oauth_api;
use crate::oauth_api::OAuthApi;
pub mod oauth_connection;
use crate::oauth_connection::OAuthConnection;

pub struct ApiData {
    pub oauth_apis: Store<OAuthApi>,
    pub oauth_connections: Store<OAuthConnection>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Kube Api Error: {0}")]
    KubeError(#[source] kube::Error),

    #[error("SerializationError: {0}")]
    SerializationError(#[source] serde_json::Error),
}
pub type Result<T, E = Error> = std::result::Result<T, E>;
