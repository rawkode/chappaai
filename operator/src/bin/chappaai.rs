use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use chappaai::{oauth_api, oauth_connection, ApiData};
use tracing::{info, warn};
use tracing_subscriber::{prelude::*, EnvFilter, Registry};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let logger = tracing_subscriber::fmt::layer().json();
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();
    let collector = Registry::default().with(logger).with(env_filter);
    tracing::subscriber::set_global_default(collector).unwrap();

    let client = kube::Client::try_default()
        .await
        .expect("Couldn't create Kubernetes client");

    let (_, oauth_api_store, oauth_api_controller) = oauth_api::Manager::new(client.clone()).await;
    let (_, oauth_connection_store, oauth_connection_controller) =
        crate::oauth_connection::Manager::new(client.clone()).await;

    let api = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:4369")
            .allowed_origin("http://127.0.0.1:4369")
            .allowed_methods(vec!["GET"])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(ApiData {
                client: client.clone(),
                oauth_apis: oauth_api_store.clone(),
                oauth_connections: oauth_connection_store.clone(),
            }))
            .service(oauth_api::api::list)
            .service(oauth_connection::api::list)
            .service(oauth_connection::api::connect)
            .service(oauth_connection::api::callback)
    })
    .bind("0.0.0.0:4370")?;

    tokio::select! {
        _ = oauth_api_controller => warn!("controller drained"),
        _ = oauth_connection_controller => warn!("controller drained"),
        _ = api.run() => info!("actix exited"),
    }

    Ok(())
}
