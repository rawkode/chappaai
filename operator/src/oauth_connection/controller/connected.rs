use super::OAuthConnection;
use crate::Error;

use kube::{
    runtime::{controller::Action, events::Recorder},
    Client,
};

use std::sync::Arc;
use tokio::time::Duration;

pub async fn connect(
    _client: Client,
    _recorder: Recorder,
    _oauth_connection: Arc<OAuthConnection>,
) -> Result<Action, Error> {
    Ok(Action::requeue(Duration::from_secs(60)))
}
