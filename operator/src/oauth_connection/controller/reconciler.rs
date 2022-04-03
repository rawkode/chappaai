use crate::kubernetes::controller;
use crate::oauth_connection::OAuthConnection;
use crate::Error;
use kube::runtime::controller::{Action, Context};
use std::sync::Arc;
use tokio::time::Duration;
use tracing::warn;

pub(super) fn error_policy(error: &Error, _: Context<controller::Data>) -> Action {
    warn!("reconcile failed: {:?}", error);
    Action::requeue(Duration::from_secs(5 * 60))
}

pub(super) async fn reconcile(
    api_service: Arc<OAuthConnection>,
    ctx: Context<controller::Data>,
) -> Result<Action, Error> {
    Ok(Action::requeue(Duration::from_secs(30 * 60)))
}
