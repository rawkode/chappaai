use super::OAuthConnection;
use crate::apiVersion;
use crate::oauth_connection::OAuthConnectionPhase;
use crate::oauth_connection::OAuthConnectionStatus;
use crate::Error;
use k8s_openapi::api::core::v1::Secret;
use kube::api::Patch;
use kube::api::PatchParams;
use kube::Client;
use kube::Resource;
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
use tracing::info;

pub async fn initializing(
    client: Client,
    recorder: Recorder,
    oauth_connection: Arc<OAuthConnection>,
) -> Result<Action, Error> {
    let name = oauth_connection.name();
    let namespace = oauth_connection.namespace().expect("Namespace is required");

    let api: Api<OAuthConnection> = Api::namespaced(client.clone(), &namespace);
    let secrets: Api<Secret> = Api::namespaced(client, &namespace);

    let _ = match oauth_connection.spec.load_client_secret(secrets).await {
        Ok(secret) => secret,
        Err(e) => {
            recorder
                .publish(Event {
                    type_: EventType::Warning,
                    reason: format!("Client Secret: Secret unavailable"),
                    note: Some("Failed to initialize".into()),
                    action: "Initializing".into(),
                    secondary: None,
                })
                .await
                .map_err(Error::KubeError)?;
            return Err(e);
        }
    };

    let new_status = Patch::Apply(json!({
        "apiVersion": apiVersion(),
        "kind": "OAuthConnection",
        "status": OAuthConnectionStatus {
            phase: Some(OAuthConnectionPhase::Disconnected),
        }
    }));

    let ps = PatchParams::apply("cntrlr").force();
    let _o = api
        .patch_status(&name, &ps, &new_status)
        .await
        .map_err(Error::KubeError)?;

    recorder
        .publish(Event {
            type_: EventType::Normal,
            reason: "Client Secret Available".into(),
            note: Some("Initialized. Moving to Disconnected".into()),
            action: "Disconnected".into(),
            secondary: None,
        })
        .await
        .map_err(Error::KubeError)?;

    info!("Reconciled Foo \"{}\" in {}", name, namespace);

    Ok(Action::requeue(Duration::from_secs(30 * 60)))
}
