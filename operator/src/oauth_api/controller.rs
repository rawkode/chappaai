use super::{OAuthApi, OAuthApiPhase, OAuthApiStatus};
use crate::{api_version, kubernetes::controller, Error};
use chrono::prelude::*;
use futures::{future::BoxFuture, FutureExt, StreamExt};
use kube::{
    api::{Api, ListParams, Patch, PatchParams, ResourceExt},
    client::Client,
    runtime::{
        controller::{Action, Context, Controller},
        events::{Event, EventType, Recorder},
        reflector::Store,
    },
    Resource,
};
use serde_json::json;
use std::sync::Arc;
use tokio::{sync::RwLock, time::Duration};
use tracing::{info, warn};

#[derive(Clone)]
pub struct Manager {
    /// Client
    pub client: kube::Client,

    /// In memory state
    state: Arc<RwLock<controller::State>>,
}

/// Example Manager that owns a Controller for Foo
impl Manager {
    /// Lifecycle initialization interface for app
    ///
    /// This returns a `Manager` that drives a `Controller` + a future to be awaited
    /// It is up to `main` to wait for the controller stream.
    pub async fn new(client: Client) -> (Self, Store<OAuthApi>, BoxFuture<'static, ()>) {
        let state = Arc::new(RwLock::new(controller::State::new(String::from("oauth-apis"))));
        let context = Context::new(controller::Data {
            client: client.clone(),
            state: state.clone(),
        });

        let api_services = Api::<OAuthApi>::all(client.clone());

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

    /// State getter
    pub async fn state(&self) -> controller::State {
        self.state.read().await.clone()
    }
}

fn error_policy(error: &Error, _: Context<controller::Data>) -> Action {
    warn!("reconcile failed: {:?}. Will try again in 5 minutes", error);
    Action::requeue(Duration::from_secs(5 * 60))
}

async fn reconcile(api_service: Arc<OAuthApi>, ctx: Context<controller::Data>) -> Result<Action, Error> {
    let client = ctx.get_ref().client.clone();
    ctx.get_ref().state.write().await.last_event = Utc::now();

    let reporter = ctx.get_ref().state.read().await.reporter.clone();
    let recorder = Recorder::new(client.clone(), reporter, api_service.object_ref(&()));

    let name = api_service.name();
    let namespace = api_service.namespace();

    let api_services: Api<OAuthApi> = match namespace {
        Some(namespace) => Api::namespaced(client, &namespace),
        None => Api::default_namespaced(client),
    };

    let new_status = Patch::Apply(json!({
        "apiVersion": api_version(),
        "kind": "OAuthApi",
        "status": OAuthApiStatus {
            phase: Some(OAuthApiPhase::Registered),
        }
    }));

    let _ = api_services
        .patch_status(&name, &PatchParams::apply("chappaai").force(), &new_status)
        .await
        .map_err(Error::KubeError)?;

    recorder
        .publish(Event {
            type_: EventType::Normal,
            action: "Registered".into(),
            secondary: None,
            reason: "Successfully registered OAuthAPI".into(),
            note: None,
        })
        .await
        .map_err(Error::KubeError)?;

    info!("Reconciled OAuthAPI: \"{}\"", name);

    Ok(Action::await_change())
}
