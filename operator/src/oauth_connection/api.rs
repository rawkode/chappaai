use std::collections::HashMap;

use crate::{
    apiVersion,
    oauth_connection::{OAuthConnectionPhase, OAuthConnectionStatus},
    ApiData,
};
use actix_web::{get, web, HttpRequest, HttpResponse};
use k8s_openapi::api::core::v1::Secret;
use kube::{
    api::{Patch, PatchParams},
    Api,
};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    RevocationUrl, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OAuthConnectionWeb {
    name: String,
    phase: String,
}

#[get("/oauth/connections")]
pub async fn list(data: web::Data<ApiData>) -> HttpResponse {
    let oauth_connections = &data.oauth_connections.state(); // <- get app_name
    let names: Vec<OAuthConnectionWeb> = oauth_connections
        .iter()
        .map(|service| {
            let meta = service.metadata.clone();

            OAuthConnectionWeb {
                name: meta.name.unwrap_or(String::from("Unknown")),
                phase: match &service.status {
                    Some(OAuthConnectionStatus { phase: Some(phase) }) => phase.into(),
                    _ => String::from("Status and phase not known"),
                },
            }
        })
        .collect();

    HttpResponse::Ok().content_type("application/json").json(names)
}

#[get("/oauth/connections/{name}")]
pub async fn connect(req: HttpRequest, path: web::Path<(String)>, data: web::Data<ApiData>) -> HttpResponse {
    let oauth_connection_name = path.into_inner();

    let oacs = data.oauth_connections.state();
    let oaas = data.oauth_apis.state();

    let oac = match oacs
        .iter()
        .find(|c| &c.metadata.name.clone().unwrap_or(String::from("Unknown")) == &oauth_connection_name)
    {
        Some(c) => c.as_ref(),
        None => return HttpResponse::NotFound().finish(),
    };

    let oaa = match oaas
        .iter()
        .find(|c| &c.metadata.name.clone().unwrap_or(String::from("Unknown")) == &oac.spec.api)
    {
        Some(c) => c.as_ref(),
        None => return HttpResponse::NotFound().finish(),
    };

    let client = kube::Client::try_default().await.expect("create client");
    let secrets: Api<Secret> = Api::default_namespaced(client);

    let (client_id, client_secret) = match oac.spec.load_client_keys(secrets).await {
        Ok(secret) => secret,
        Err(e) => return HttpResponse::NotFound().finish(),
    };

    let auth_url = AuthUrl::new(oaa.get_authorization_url()).unwrap();
    let token_url = TokenUrl::new(oaa.get_token_url()).unwrap();

    let redirect_url = format!("http://localhost:3000/oauth/callback/{}", oauth_connection_name);

    let oauth_client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap());

    let oauth_client = oauth_client.authorize_url(|| CsrfToken::new(String::from("abc")));

    let oauth_client = oac.spec.scopes.iter().fold(oauth_client, |mut client, scope| {
        client.add_scope(Scope::new(scope.clone()))
    });

    let (auth_url, csrf_token) = oauth_client.url();

    HttpResponse::TemporaryRedirect()
        .header("Location", auth_url.to_string())
        .finish()
}

#[derive(Clone, Debug, Deserialize)]
pub struct OAuthResponse {
    code: String,
    state: String,
}

#[get("/oauth/callback/{name}")]
pub async fn callback(
    result: web::Query<OAuthResponse>,
    path: web::Path<(String)>,
    data: web::Data<ApiData>,
) -> HttpResponse {
    // let csrf_check = CsrfToken::new(result.state.clone());
    let auth = AuthorizationCode::new(result.code.clone());

    let oauth_connection_name = path.into_inner();

    let oacs = data.oauth_connections.state();
    let oaas = data.oauth_apis.state();

    let oac = match oacs
        .iter()
        .find(|c| &c.metadata.name.clone().unwrap_or(String::from("Unknown")) == &oauth_connection_name)
    {
        Some(c) => c.as_ref(),
        None => return HttpResponse::NotFound().finish(),
    };

    let oaa = match oaas
        .iter()
        .find(|c| &c.metadata.name.clone().unwrap_or(String::from("Unknown")) == &oac.spec.api)
    {
        Some(c) => c.as_ref(),
        None => return HttpResponse::NotFound().finish(),
    };

    let client = kube::Client::try_default().await.expect("create client");
    let secrets: Api<Secret> = Api::default_namespaced(client);

    let (client_id, client_secret) = match oac.spec.load_client_keys(secrets).await {
        Ok(secret) => secret,
        Err(e) => return HttpResponse::NotFound().finish(),
    };

    let auth_url = AuthUrl::new(oaa.get_authorization_url()).unwrap();
    let token_url = TokenUrl::new(oaa.get_token_url()).unwrap();

    let redirect_url = format!("http://localhost:3000/oauth/callback/{}", oauth_connection_name);

    let oauth_client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap());

    let token = match oauth_client
        .exchange_code(auth)
        .request_async(oauth2::reqwest::async_http_client)
        .await
    {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Failed: {:?}", e));
        }
    };

    let a = token.access_token().secret();

    HttpResponse::Ok().body(a.clone())
}
