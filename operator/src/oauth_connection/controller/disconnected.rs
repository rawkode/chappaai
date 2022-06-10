use super::OAuthConnection;
use crate::apiVersion;
use crate::oauth_connection::OAuthConnectionPhase;
use crate::oauth_connection::OAuthConnectionStatus;
use crate::Error;
use k8s_openapi::api::core::v1::Secret;
use kube::api::Patch;
use kube::api::PatchParams;
use kube::Client;
use kube::ResourceExt;
use kube::{
    api::Api,
    runtime::{
        controller::Action,
        events::{Event, EventType, Recorder},
    },
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
            Api::namespaced(client.clone(), &namespace),
            Api::namespaced(client, &namespace),
        ),
        None => (
            Api::default_namespaced(client.clone()),
            Api::default_namespaced(client),
        ),
    };

    let (client_id, client_secret) = match oauth_connection.spec.load_client_keys(secrets).await {
        Ok(secret) => secret,
        Err(e) => {
            let new_status = Patch::Apply(json!({
                "apiVersion": apiVersion(),
                "kind": "OAuthConnection",
                "status": OAuthConnectionStatus {
                    phase: Some(OAuthConnectionPhase::Initializing),
                }
            }));

            let ps = PatchParams::apply("cntrlr").force();
            let _o = api
                .patch_status(&name, &ps, &new_status)
                .await
                .map_err(Error::KubeError)?;

            recorder
                .publish(Event {
                    type_: EventType::Warning,
                    reason: format!("❌ Client ID/Secret unavailable"),
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
