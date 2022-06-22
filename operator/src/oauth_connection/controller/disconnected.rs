use super::OAuthConnection;
use crate::{
    api_version,
    oauth_connection::{OAuthConnectionPhase, OAuthConnectionStatus},
    Error,
};
use k8s_openapi::api::core::v1::Secret;
use kube::{
    api::{Api, Patch, PatchParams},
    runtime::{
        controller::Action,
        events::{Event, EventType, Recorder},
    },
    Client, ResourceExt,
};
use serde_json::json;
use std::sync::Arc;
use tokio::time::Duration;

pub async fn disconnected(
    client: Client,
    recorder: Recorder,
    oauth_connection: Arc<OAuthConnection>,
) -> Result<Action, Error> {
    let name = oauth_connection.name();
    let namespace = oauth_connection.namespace();

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

    let (_client_id, _client_secret) = match oauth_connection.load_client_keys(secrets).await {
        Ok(secret) => secret,
        Err(_e) => {
            let new_status = Patch::Apply(json!({
                "apiVersion": api_version(),
                "kind": "OAuthConnection",
                "status": OAuthConnectionStatus {
                    phase: Some(OAuthConnectionPhase::Initializing),
                }
            }));

            let patch_params = PatchParams::apply("chappaai").force();
            let _ = api
                .patch_status(&name, &patch_params, &new_status)
                .await
                .map_err(Error::KubeError)?;

            recorder
                .publish(Event {
                    type_: EventType::Warning,
                    reason: "❌ Client ID/Secret unavailable".to_string(),
                    note: Some("Failed to initialize".into()),
                    action: "Initializing".into(),
                    secondary: None,
                })
                .await
                .map_err(Error::KubeError)?;

            return Ok(Action::requeue(Duration::from_secs(60)));
        }
    };

    Ok(Action::requeue(Duration::from_secs(60)))
}
