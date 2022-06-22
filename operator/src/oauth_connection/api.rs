use std::{
    collections::BTreeMap,
    time::{Duration, SystemTime},
};

use super::OAuthConnection;
use crate::{
    api_version,
    oauth_connection::{OAuthConnectionPhase, OAuthConnectionStatus},
    ApiData,
};
use actix_web::{get, web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use k8s_openapi::api::core::v1::Secret;
use kube::{
    api::{Patch, PatchParams, PostParams},
    core::ObjectMeta,
    Api, ResourceExt,
};
use kube_client::Error;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
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
                name: meta.name.unwrap_or_else(|| String::from("Unknown")),
                phase: match &service.status {
                    Some(OAuthConnectionStatus {
                        phase: Some(phase),
                        secret_name: None,
                        expires_at: None,
                    }) => phase.into(),
                    _ => String::from("Status and phase not known"),
                },
            }
        })
        .collect();

    HttpResponse::Ok().content_type("application/json").json(names)
}

#[get("/oauth/connections/{name}")]
pub async fn connect(
    query: web::Query<OAuthRequest>,
    path: web::Path<String>,
    data: web::Data<ApiData>,
) -> HttpResponse {
    let oauth_connection_name = path.into_inner();

    let oacs = data.oauth_connections.state();
    let oaas = data.oauth_apis.state();

    let oac = match oacs
        .iter()
        .find(|c| c.metadata.name.clone().unwrap_or_else(|| String::from("Unknown")) == oauth_connection_name)
    {
        Some(c) => c.as_ref(),
        None => return HttpResponse::NotFound().finish(),
    };

    let oaa = match oaas
        .iter()
        .find(|c| c.metadata.name.clone().unwrap_or_else(|| String::from("Unknown")) == oac.spec.api)
    {
        Some(c) => c.as_ref(),
        None => return HttpResponse::NotFound().finish(),
    };

    let client = data.client.clone();
    let secrets: Api<Secret> = Api::default_namespaced(client);

    let (client_id, client_secret) = match oac.load_client_keys(secrets).await {
        Ok(secret) => secret,
        Err(_e) => return HttpResponse::NotFound().finish(),
    };

    let auth_url = AuthUrl::new(oaa.get_authorization_url()).unwrap();
    let token_url = TokenUrl::new(oaa.get_token_url()).unwrap();

    let oauth_client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(query.redirect_url.clone()).unwrap());

    let oauth_client = oauth_client.authorize_url(|| CsrfToken::new(String::from("abc")));

    let oauth_client = oac.spec.scopes.iter().fold(oauth_client, |client, scope| {
        client.add_scope(Scope::new(scope.clone()))
    });

    let (auth_url, _csrf_token) = oauth_client.url();

    HttpResponse::TemporaryRedirect()
        .append_header(("Location", auth_url.to_string()))
        .finish()
}

#[derive(Deserialize)]
pub struct OAuthRequest {
    redirect_url: String,
}

#[derive(Deserialize)]
pub struct OAuthResponse {
    code: String,
    #[allow(dead_code)]
    state: String,
    redirect_url: String,
}

#[get("/oauth/callback/{name}")]
pub async fn callback(
    query: web::Query<OAuthResponse>,
    path: web::Path<String>,
    data: web::Data<ApiData>,
) -> HttpResponse {
    // let csrf_check = CsrfToken::new(result.state.clone());
    let auth = AuthorizationCode::new(query.code.clone());

    let oauth_connection_name = path.into_inner();

    let oacs = data.oauth_connections.state();
    let oaas = data.oauth_apis.state();

    let oac = match oacs
        .iter()
        .find(|c| c.metadata.name.clone().unwrap_or_else(|| String::from("Unknown")) == oauth_connection_name)
    {
        Some(c) => c.as_ref(),
        None => return HttpResponse::NotFound().finish(),
    };

    let name = oac.name();
    let namespace = oac.namespace();

    let oaa = match oaas
        .iter()
        .find(|c| c.metadata.name.clone().unwrap_or_else(|| String::from("Unknown")) == oac.spec.api)
    {
        Some(c) => c.as_ref(),
        None => return HttpResponse::NotFound().finish(),
    };

    let client = data.client.clone();

    let (api, secrets): (Api<OAuthConnection>, Api<Secret>) = match &namespace {
        Some(namespace) => (
            Api::namespaced(client.clone(), namespace),
            Api::namespaced(client, namespace),
        ),
        None => (
            Api::default_namespaced(client.clone()),
            Api::default_namespaced(client),
        ),
    };

    let (client_id, client_secret) = match oac.load_client_keys(secrets.clone()).await {
        Ok(secret) => secret,
        Err(_e) => return HttpResponse::NotFound().finish(),
    };

    let auth_url = AuthUrl::new(oaa.get_authorization_url()).unwrap();
    let token_url = TokenUrl::new(oaa.get_token_url()).unwrap();

    let oauth_client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(query.redirect_url.clone()).unwrap());

    let token = match oauth_client
        .exchange_code(auth)
        .request_async(oauth2::reqwest::async_http_client)
        .await
    {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::Unauthorized().body(format!("Failed: {:?}", e));
        }
    };

    let a = token.access_token().secret();

    let secret_name = format!("chappaai-{}", name);

    let mut string_data: BTreeMap<String, String> = BTreeMap::new();
    string_data.insert("accessToken".to_string(), a.to_string());

    let new_secret = Secret {
        metadata: ObjectMeta {
            name: Some(secret_name.clone()),
            namespace: namespace.clone(),
            ..Default::default()
        },
        immutable: Some(false),
        string_data: Some(string_data),
        ..Default::default()
    };

    let post_params = PostParams {
        dry_run: false,
        ..Default::default()
    };

    match secrets.create(&post_params, &new_secret).await {
        Ok(_) => true,
        Err(e) => {
            println!("Failed: {:?}", e);

            false
        }
    };

    let duration = token.expires_in().unwrap_or(Duration::from_secs(0));
    let time = SystemTime::now() + duration;
    let datetime: DateTime<Utc> = time.into();

    let new_status = Patch::Apply(json!({
                "apiVersion": api_version(),
                "kind": "OAuthConnection",
                "status": OAuthConnectionStatus {
                  phase: Some(OAuthConnectionPhase::Connected),
                  secret_name: Some(secret_name.clone()),
                  expires_at: Some(datetime.to_rfc3339()),
                }
    }));

    let patch_params = PatchParams::apply("chappaai").force();

    let _ = match api
        .patch_status(&name, &patch_params, &new_status)
        .await
        .map_err(crate::Error::KubeError)
    {
        Ok(_) => true,
        Err(e) => {
            println!("Failed: {:?}", e);

            false
        }
    };

    HttpResponse::Ok().body(a.clone())
}
