use crate::ApiData;
use actix_web::{get, web, HttpResponse};

#[get("/oauth/connections")]
pub async fn list(data: web::Data<ApiData>) -> HttpResponse {
    let oauth_connections = &data.oauth_connections.state(); // <- get app_name
    let names: Vec<String> = oauth_connections
        .iter()
        .map(|service| {
            let meta = service.metadata.clone();

            meta.name.unwrap_or(String::from("Unknown"))
        })
        .collect();

    HttpResponse::Ok().content_type("application/json").json(names)
}
