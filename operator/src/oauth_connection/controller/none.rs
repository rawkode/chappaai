use super::{OAuthConnection, OAuthConnectionPhase, OAuthConnectionStatus};
use crate::{api_version, Error};
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
use tracing::info;

pub async fn none(
    client: Client,
    recorder: Recorder,
    oauth_connection: Arc<OAuthConnection>,
) -> Result<Action, Error> {
    let name = oauth_connection.name();
    let namespace = oauth_connection.namespace();

    let api: Api<OAuthConnection> = match &namespace {
        Some(namespace) => Api::namespaced(client, namespace),
        None => Api::default_namespaced(client),
    };

    let new_status = Patch::Apply(json!({
        "apiVersion": api_version(),
        "kind": "OAuthConnection",
        "status": OAuthConnectionStatus {
            phase: Some(OAuthConnectionPhase::Initializing),
                        expires_at: None,
                        secret_name: None,
        }
    }));

    let patch_params = PatchParams::apply("chappaai").force();
    let _ = api
        .patch_status(&name, &patch_params, &new_status)
        .await
        .map_err(Error::KubeError)?;

    recorder
        .publish(Event {
            type_: EventType::Normal,
            reason: "Unknown State".into(),
            note: Some("Bootstrapped. Moving to Initializing".into()),
            action: "Bootstrapping".into(),
            secondary: None,
        })
        .await
        .map_err(Error::KubeError)?;

    info!("Reconciled OAuthConnection {}", name);

    Ok(Action::await_change())
}
