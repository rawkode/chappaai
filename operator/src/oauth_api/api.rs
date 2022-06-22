use crate::ApiData;
use actix_web::{get, web, HttpResponse};

#[get("/oauth/apis")]
async fn list(data: web::Data<ApiData>) -> HttpResponse {
    let oauth_apis = &data.oauth_apis.state();

    let oauth_api_names: Vec<String> = oauth_apis
        .iter()
        .map(|service| {
            let meta = &service.metadata;
            meta.name.clone().unwrap_or(String::from("Unknown"))
        })
        .collect();

    HttpResponse::Ok().json(oauth_api_names)
}
