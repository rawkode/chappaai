use super::{OAuthConnection, OAuthConnectionPhase, OAuthConnectionStatus};
use crate::{kubernetes::controller, Error};
use chrono::prelude::*;
use futures::{future::BoxFuture, FutureExt, StreamExt};
use kube::{
    api::{Api, ListParams},
    client::Client,
    runtime::{
        controller::{Action, Context, Controller},
        events::Recorder,
        reflector::Store,
    },
    Resource,
};
use std::sync::Arc;
use tokio::{sync::RwLock, time::Duration};
use tracing::warn;

// Controller States
mod none;
use none::none;
mod initializing;
use initializing::initializing;
mod disconnected;
use disconnected::disconnected;
mod connected;
use connected::connect;

#[derive(Clone)]
pub struct Manager {
    /// Client
    client: kube::Client,

    /// In memory state
    state: Arc<RwLock<controller::State>>,
}

impl Manager {
    pub async fn new(client: Client) -> (Self, Store<OAuthConnection>, BoxFuture<'static, ()>) {
        let state = Arc::new(RwLock::new(controller::State::new(String::from(
            "oauth-connections",
        ))));

        let context = Context::new(controller::Data {
            client: client.clone(),
            state: state.clone(),
        });

        let api_services = Api::<OAuthConnection>::namespaced(client.clone(), "default");

        // Ensure the CRD's are installed and we have access to list them
        api_services
            .list(&ListParams::default().limit(1))
            .await
            .expect("Unable to access OAuthConnection's within the current namespace");

        // All good. Start controller and return its future.
        let drainer = Controller::new(api_services, ListParams::default());

        let store = drainer.store();

        let drainer = drainer
            .run(reconcile, error_policy, context)
            .filter_map(|x| async move { std::result::Result::ok(x) })
            .for_each(|_| futures::future::ready(()))
            .boxed();

        (Self { client, state }, store, drainer)
    }

    /// Client getter
    pub async fn client(&self) -> kube::Client {
        self.client.clone()
    }

    /// State getter
    pub async fn state(&self) -> controller::State {
        self.state.read().await.clone()
    }
}

fn error_policy(error: &Error, _: Context<controller::Data>) -> Action {
    warn!("reconcile failed: {:?}", error);
    Action::requeue(Duration::from_secs(5 * 60))
}

async fn reconcile(
    oauth_connection: Arc<OAuthConnection>,
    ctx: Context<controller::Data>,
) -> Result<Action, Error> {
    let client = ctx.get_ref().client.clone();
    ctx.get_ref().state.write().await.last_event = Utc::now();

    let reporter = ctx.get_ref().state.read().await.reporter.clone();
    let recorder = Recorder::new(client.clone(), reporter.clone(), oauth_connection.object_ref(&()));

    match &oauth_connection.status {
        Some(status) => match &status.phase {
            Some(phase) => match &phase {
                OAuthConnectionPhase::Initializing => initializing(client, recorder, oauth_connection).await,
                OAuthConnectionPhase::Disconnected => disconnected(client, recorder, oauth_connection).await,
                OAuthConnectionPhase::Connected => connect(client, recorder, oauth_connection).await,
            },
            None => none(client, recorder, oauth_connection).await,
        },
        None => none(client, recorder, oauth_connection).await,
    }
}
