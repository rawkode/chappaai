use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, SystemTime},
};

use super::OAuthConnection;
use crate::{
    api_version,
    oauth_connection::{OAuthConnectionPhase, OAuthConnectionStatus},
    ApplicationState, OAuthApi, Result,
};

use axum::{
    extract::{Path, Query},
    response::{IntoResponse, Redirect},
    Extension, Json,
};
use chrono::{DateTime, Utc};
use hyper::StatusCode;
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
pub struct OAuthConnectionWeb {
    name: String,
    phase: String,
}

pub async fn list(
    Extension(state): Extension<Arc<ApplicationState>>,
) -> Result<Json<Vec<OAuthConnectionWeb>>> {
    let oauth_connections = &state.oauth_connections.state(); // <- get app_name
    let names: Vec<OAuthConnectionWeb> = oauth_connections
        .iter()
        .map(|service| {
            let meta = service.metadata.clone();

            OAuthConnectionWeb {
                name: meta.name.unwrap_or_else(|| String::from("Unknown")),
                phase: match &service.status {
                    Some(OAuthConnectionStatus {
                        phase: Some(phase),
                        secret_name: _,
                        expires_at: _,
                    }) => phase.into(),
                    _ => String::from("Status and phase not known"),
                },
            }
        })
        .collect();

    Ok(Json(names))
}

pub async fn connect(
    Query(query): Query<OAuthRequest>,
    Path(name): Path<String>,
    Extension(state): Extension<Arc<ApplicationState>>,
) -> impl IntoResponse {
    let oauth_connection_name = name;

    let (oac, oaa) = match oauth_connection_and_api(
        oauth_connection_name,
        state.oauth_connections.state(),
        state.oauth_apis.state(),
    ) {
        Some(result) => result,
        None => {
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    let client = state.client.clone();
    let secrets: Api<Secret> = Api::default_namespaced(client.clone());

    let oauth_client = match oauth_basic_client(secrets.clone(), &oac, &oaa, query.redirect_url.clone()).await
    {
        Ok(c) => c,
        Err(error) => {
            println!("Returning 404 because: {:?}", error);
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    let oauth_client = oauth_client.authorize_url(|| CsrfToken::new(String::from("abc")));

    let oauth_client = oac.spec.scopes.iter().fold(oauth_client, |client, scope| {
        client.add_scope(Scope::new(scope.clone()))
    });

    let (auth_url, _csrf_token) = oauth_client.url();

    Redirect::temporary(auth_url.as_ref()).into_response()
}

async fn oauth_basic_client(
    secrets: Api<Secret>,
    oac: &OAuthConnection,
    oaa: &OAuthApi,
    redirect_url: String,
) -> Result<oauth2::basic::BasicClient> {
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

pub async fn callback(
    Query(query): Query<OAuthResponse>,
    Path(name): Path<String>,
    Extension(state): Extension<Arc<ApplicationState>>,
) -> impl IntoResponse {
    // let csrf_check = CsrfToken::new(result.state.clone());
    let auth = AuthorizationCode::new(query.code.clone());

    let oauth_connection_name = name;

    let (oac, oaa) = match oauth_connection_and_api(
        oauth_connection_name.clone(),
        state.oauth_connections.state(),
        state.oauth_apis.state(),
    ) {
        Some(result) => result,
        None => {
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    let name = oac.name();
    let namespace = oac.namespace();

    let client = state.client.clone();

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
        Err(error) => {
            println!("Returning 404 because: {:?}", error);
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    let token = match oauth_client
        .exchange_code(auth)
        .request_async(oauth2::reqwest::async_http_client)
        .await
    {
        Ok(t) => t,
        Err(e) => {
            return (StatusCode::UNAUTHORIZED, format!("Failed: {:?}", e)).into_response();
        }
    };

    let secret_name = format!("chappaai-{}", name);
    let owner_ref = oac.controller_owner_ref(&()).unwrap();

    let mut string_data: BTreeMap<String, String> = BTreeMap::new();
    string_data.insert(
        "accessToken".to_string(),
        token.access_token().secret().to_string(),
    );

    if let Some(refresh_token) = token.refresh_token() {
        string_data.insert("refreshToken".to_string(), refresh_token.secret().to_string());
    }

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

    "Connected".into_response()
}
