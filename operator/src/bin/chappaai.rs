use actix_web::HttpServer;
use actix_web::{middleware, web, web::Data, App};
use chappaai::oauth_api;
use chappaai::oauth_connection;
use chappaai::ApiData;
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

    let (oauth_api_manager, oauth_api_store, oauth_api_controller) = oauth_api::Manager::new().await;
    let (_, oauth_connection_store, oauth_connection_controller) =
        crate::oauth_connection::Manager::new().await;

    let oauth_api_process = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(oauth_api_manager.clone()))
            .wrap(middleware::Logger::default().exclude("/health"))
            .service(oauth_api::index)
            .service(oauth_api::health)
    })
    .bind("0.0.0.0:8080")
    .expect("Can not bind to 0.0.0.0:8080")
    .shutdown_timeout(5);

    let s2 = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(ApiData {
                oauth_apis: oauth_api_store.clone(),
                oauth_connections: oauth_connection_store.clone(),
            }))
            .service(oauth_api::api::list)
            .service(oauth_connection::api::list)
            .service(oauth_connection::api::connect)
    })
    .bind("0.0.0.0:7979")?;

    tokio::select! {
        _ = oauth_api_controller => warn!("controller drained"),
        _ = oauth_connection_controller => warn!("controller drained"),
        _ = oauth_api_process.run() => info!("actix exited"),
        _ = s2.run() => info!("actix exited"),
    }

    Ok(())
}
