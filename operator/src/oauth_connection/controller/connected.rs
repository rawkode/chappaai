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

pub async fn connect(
    client: Client,
    recorder: Recorder,
    oauth_connection: Arc<OAuthConnection>,
) -> Result<Action, Error> {
    Ok(Action::requeue(Duration::from_secs(60)))
}
