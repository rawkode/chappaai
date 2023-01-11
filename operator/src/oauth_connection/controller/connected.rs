use super::OAuthConnection;
use crate::OAuthApi;
use crate::{kubernetes::controller::state_change, oauth_connection::OAuthConnectionPhase, Error};
use chrono::Utc;
use k8s_openapi::api::core::v1::Secret;
use kube::{
    api::DeleteParams,
    runtime::{
        controller::Action,
        events::{Event, EventType, Recorder},
    },
    Api, Client, ResourceExt,
};
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RefreshToken, TokenUrl};
use std::{sync::Arc, time::Duration};

pub async fn connect(
    client: Client,
    recorder: Recorder,
    oauth_connection: Arc<OAuthConnection>,
) -> Result<Action, Error> {
    let name = oauth_connection.name();
    let namespace = oauth_connection.namespace();

    let (_api, oauth_apis, secrets): (Api<OAuthConnection>, Api<OAuthApi>, Api<Secret>) = match &namespace {
        Some(namespace) => (
            Api::namespaced(client.clone(), namespace),
            Api::namespaced(client.clone(), namespace),
            Api::namespaced(client, namespace),
        ),
        None => (
            Api::default_namespaced(client.clone()),
            Api::default_namespaced(client.clone()),
            Api::default_namespaced(client),
        ),
    };

    // We lost our status, roll us back to Disconnected
    if oauth_connection.status.is_none() {
        match secrets
            .delete(format!("chappaai-{}", name).as_str(), &DeleteParams::default())
            .await
        {
            Ok(_) => (),
            Err(error) => {
                let _ = recorder
                    .publish(Event {
                        type_: EventType::Warning,
                        reason: "❌ Failed to delete secret for disconnected connection".to_string(),
                        note: Some(format!("Will try again in 1 minute: {}", error)),
                        action: "Disconnected Cleanup".into(),
                        secondary: None,
                    })
                    .await;

                return Ok(Action::requeue(Duration::from_secs(300)));
            }
        }

        match state_change(
            &oauth_connection,
            &recorder,
            Event {
                type_: EventType::Warning,
                reason: "❌ Client ID/Secret unavailable".to_string(),
                note: Some("Failed to initialize".into()),
                action: "Initializing".into(),
                secondary: None,
            },
            OAuthConnectionPhase::Disconnected,
        )
        .await
        {
            Ok(_) => (),
            Err(error) => {
                println!(
                    "Failed to record state change. Bad things are happening: {}",
                    error
                );
                println!("Trying again in 1 minute");

                return Ok(Action::requeue(Duration::from_secs(60)));
            }
        }
    }

    let status = oauth_connection.status.as_ref().unwrap();
    if status.expires_at.is_none() {
        // What even?
        return Ok(Action::requeue(Duration::from_secs(60)));
    }

    let status_expires_at = status.expires_at.as_ref().unwrap();

    let expiry_time = match chrono::DateTime::parse_from_rfc3339(status_expires_at) {
        Ok(expires_at) => expires_at,
        Err(error) => {
            println!("Couldn't parse expiry time: {}", error);
            println!("Trying again in 1 minute");
            return Ok(Action::requeue(Duration::from_secs(60)));
        }
    };

    // Renew within 3 hours, that gives us some resiliency in-case our
    // pod disappears for a while
    if expiry_time.naive_utc() - Utc::now().naive_utc() >= chrono::Duration::seconds(10800) {
        println!("Token has at-least a three hours left, no need to refresh");
        return Ok(Action::requeue(Duration::from_secs(3600)));
    }

    // Renew
    let secret_tokens = secrets
        .get(format!("chappaai-{}", name).as_str())
        .await?
        .data
        .expect("Couldn't access data on secret");

    let oauth_api = oauth_apis.get(&oauth_connection.spec.api).await?;
    let (client_id, client_secret) = oauth_connection.load_client_keys(secrets).await?;

    let auth_url = AuthUrl::new(oauth_api.get_authorization_url()).unwrap();
    let token_url = TokenUrl::new(oauth_api.get_token_url()).unwrap();

    let oauth_client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        auth_url,
        Some(token_url),
    );

    if let Some(ref refresh_token) = secret_tokens.get("refreshToken") {
        let token = match oauth_client
            .exchange_refresh_token(&RefreshToken::new(String::from_utf8(refresh_token.0).unwrap()))
            .request_async(oauth2::reqwest::async_http_client)
            .await
        {
            Ok(token) => token,
            Err(_) => {
                println!("BROKEN");
                return Ok(Action::requeue(Duration::from_secs(3600)));
            }
        };
    }

    Ok(Action::requeue(Duration::from_secs(3600)))
}
