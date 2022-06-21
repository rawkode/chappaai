use crate::ApiData;
use actix_web::{get, web, HttpResponse};

#[get("/oauth/apis")]
pub async fn list(data: web::Data<ApiData>) -> HttpResponse {
    let oauth_apis = &data.oauth_apis.state(); // <- get app_name
    let names: Vec<String> = oauth_apis
        .iter()
        .map(|service| {
            let meta = service.metadata.clone();

            meta.name.unwrap_or_else(|| String::from("Unknown"))
        })
        .collect();

    HttpResponse::Ok().content_type("application/json").json(names)
}
