use crate::ApiData;
use actix_web::{get, web, HttpResponse};
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, RevocationUrl, TokenUrl};

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

#[get("/oauth/connections/{name}")]
pub async fn connect(data: web::Data<ApiData>) -> HttpResponse {
    // let google_client_id = ClientId::new(env::var("GOOGLE_CLIENT_ID"));
    // let google_client_secret = ClientSecret::new(env::var("GOOGLE_CLIENT_SECRET"));
    // let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string());
    // let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string());

    // // Set up the config for the Google OAuth2 process.
    // let client = BasicClient::new(
    //     google_client_id,
    //     Some(google_client_secret),
    //     auth_url,
    //     Some(token_url),
    // )
    // // This example will be running its own server at localhost:8080.
    // // See below for the server implementation.
    // .set_redirect_uri(RedirectUrl::new("http://localhost:8080".to_string()).expect("Invalid redirect URL"))
    // // Google supports OAuth 2.0 Token Revocation (RFC-7009)
    // .set_revocation_uri(
    //     RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
    //         .expect("Invalid revocation endpoint URL"),
    // );

    HttpResponse::Ok().content_type("application/json").json("")
}
