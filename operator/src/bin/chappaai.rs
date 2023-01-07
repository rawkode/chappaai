use std::{net::SocketAddr, sync::Arc};

use axum::{routing::get, Extension, Router};
use chappaai::{
    oauth_api::{self},
    oauth_connection::{self},
    ApplicationState, Result,
};

use tracing_subscriber::{prelude::*, EnvFilter, Registry};

#[tokio::main]
async fn main() -> Result<()> {
    let logger = tracing_subscriber::fmt::layer().json();
    let env_filter = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    let collector = Registry::default().with(logger).with(env_filter);
    tracing::subscriber::set_global_default(collector)?;

    let client = kube::Client::try_default().await?;

    let (_, oauth_api_store, _oauth_api_controller) = oauth_api::Manager::new(client.clone()).await;
    let (_, oauth_connection_store, _oauth_connection_controller) =
        crate::oauth_connection::Manager::new(client.clone()).await;

    let address = SocketAddr::from(([0, 0, 0, 0], 4640));

    let application_state = Arc::new(ApplicationState {
        client,
        oauth_apis: oauth_api_store,
        oauth_connections: oauth_connection_store,
    });

    let router = Router::new()
        .route_service("/oauth/apis", get(oauth_api::api::list))
        .route_service("/oauth/connections", get(oauth_connection::api::list))
        .route_service("/oauth/connections/:name", get(oauth_connection::api::connect))
        .route_service("/oauth/callback/:name", get(oauth_connection::api::callback))
        .layer(Extension(application_state));

    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
