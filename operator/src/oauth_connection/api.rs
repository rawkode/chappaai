use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, SystemTime},
};

use super::OAuthConnection;
use crate::{
    api_version,
    oauth_connection::{OAuthConnectionPhase, OAuthConnectionStatus},
    ApiData, Error, OAuthApi,
};
use actix_web::{get, web, HttpResponse};
use chrono::{DateTime, Utc};
use k8s_openapi::api::core::v1::Secret;
use kube::{
    api::{Patch, PatchParams},
    core::ObjectMeta,
    Api, Resource, ResourceExt,
};

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

    let (oac, oaa) = match oauth_connection_and_api(
        oauth_connection_name,
        data.oauth_connections.state(),
        data.oauth_apis.state(),
    ) {
        Some(result) => result,
        None => return HttpResponse::NotFound().finish(),
    };

    let client = data.client.clone();
    let secrets: Api<Secret> = Api::default_namespaced(client.clone());

    let oauth_client = match oauth_basic_client(secrets.clone(), &oac, &oaa, query.redirect_url.clone()).await
    {
        Ok(c) => c,
        Err(_) => return HttpResponse::NotFound().finish(),
    };

    let oauth_client = oauth_client.authorize_url(|| CsrfToken::new(String::from("abc")));

    let oauth_client = oac.spec.scopes.iter().fold(oauth_client, |client, scope| {
        client.add_scope(Scope::new(scope.clone()))
    });

    let (auth_url, _csrf_token) = oauth_client.url();

    HttpResponse::TemporaryRedirect()
        .append_header(("Location", auth_url.to_string()))
        .finish()
}

async fn oauth_basic_client(
    secrets: Api<Secret>,
    oac: &OAuthConnection,
    oaa: &OAuthApi,
    redirect_url: String,
) -> Result<oauth2::basic::BasicClient, Error> {
    let (client_id, client_secret) = oac.load_client_keys(secrets).await?;

    let auth_url = AuthUrl::new(oaa.get_authorization_url()).unwrap();
    let token_url = TokenUrl::new(oaa.get_token_url()).unwrap();

    Ok(BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap()))
}

fn oauth_connection_and_api(
    oauth_connection_name: String,
    oacs: Vec<Arc<OAuthConnection>>,
    oaas: Vec<Arc<OAuthApi>>,
) -> Option<(OAuthConnection, OAuthApi)> {
    let oac: &OAuthConnection = match oacs
        .iter()
        .find(|c| c.metadata.name.clone().unwrap_or_else(|| String::from("Unknown")) == oauth_connection_name)
    {
        Some(c) => c,
        None => return None,
    };

    let oaa: &OAuthApi = match oaas
        .iter()
        .find(|c| c.metadata.name.clone().unwrap_or_else(|| String::from("Unknown")) == oac.spec.api)
    {
        Some(c) => c,
        None => return None,
    };

    Some((oac.to_owned(), oaa.to_owned()))
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

    let (oac, oaa) = match oauth_connection_and_api(
        oauth_connection_name.clone(),
        data.oauth_connections.state(),
        data.oauth_apis.state(),
    ) {
        Some(result) => result,
        None => return HttpResponse::NotFound().finish(),
    };

    let name = oac.name();
    let namespace = oac.namespace();

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

    let oauth_client = match oauth_basic_client(secrets.clone(), &oac, &oaa, query.redirect_url.clone()).await
    {
        Ok(c) => c,
        Err(_) => return HttpResponse::NotFound().finish(),
    };

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

    let secret_name = format!("chappaai-{}", name);
    let owner_ref = oac.controller_owner_ref(&()).unwrap();

    let mut string_data: BTreeMap<String, String> = BTreeMap::new();
    string_data.insert(
        "accessToken".to_string(),
        token.access_token().secret().to_string(),
    );

    let new_secret = Secret {
        metadata: ObjectMeta {
            name: Some(secret_name.clone()),
            namespace: namespace.clone(),
            owner_references: Some(vec![owner_ref]),
            ..ObjectMeta::default()
        },
        immutable: Some(false),
        string_data: Some(string_data),
        ..Secret::default()
    };

    let _ = match secrets
        .patch(
            secret_name.clone().as_str(),
            &PatchParams::apply("chappaai"),
            &Patch::Apply(new_secret),
        )
        .await
    {
        Ok(_) => true,
        Err(e) => {
            panic!("Failed: {:?}", e);
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

    HttpResponse::Ok().body("Connected")
}
