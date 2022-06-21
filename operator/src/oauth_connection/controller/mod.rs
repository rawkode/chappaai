use super::{OAuthConnection, OAuthConnectionPhase, OAuthConnectionStatus};
use crate::{kubernetes::controller, Error};
use actix_web::{get, web::Data as WebData, HttpRequest, HttpResponse, Responder};
use chrono::prelude::*;
use futures::{future::BoxFuture, FutureExt, StreamExt};
use kube::{
    api::{Api, ListParams, ResourceExt},
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

mod none;
use none::none;
mod initializing;
use initializing::initializing;
mod disconnected;
use disconnected::disconnected;

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
            },
            None => none(client, recorder, oauth_connection).await,
        },
        None => none(client, recorder, oauth_connection).await,
    }
}

fn error_policy(error: &Error, _: Context<controller::Data>) -> Action {
    warn!("reconcile failed: {:?}", error);
    Action::requeue(Duration::from_secs(5 * 60))
}

#[derive(Clone)]
pub struct Manager {
    /// In memory state
    state: Arc<RwLock<controller::State>>,
}

/// Example Manager that owns a Controller for Foo
impl Manager {
    /// Lifecycle initialization interface for app
    ///
    /// This returns a `Manager` that drives a `Controller` + a future to be awaited
    /// It is up to `main` to wait for the controller stream.
    pub async fn new() -> (Self, Store<OAuthConnection>, BoxFuture<'static, ()>) {
        let client = Client::try_default().await.expect("create client");
        let state = Arc::new(RwLock::new(controller::State::new(String::from("oauth-api"))));
        let context = Context::new(controller::Data {
            client: client.clone(),
            state: state.clone(),
        });

        let api_services = Api::<OAuthConnection>::all(client);

        // Ensure CRD is installed before loop-watching
        api_services
            .list(&ListParams::default().limit(1))
            .await
            .expect("is the crd installed? please run: cargo run --bin crdgen | kubectl apply -f -");

        // All good. Start controller and return its future.
        let drainer = Controller::new(api_services, ListParams::default());

        let store = drainer.store();

        let drainer = drainer
            .run(reconcile, error_policy, context)
            .filter_map(|x| async move { std::result::Result::ok(x) })
            .for_each(|_| futures::future::ready(()))
            .boxed();

        (Self { state }, store, drainer)
    }

    /// State getter
    pub async fn state(&self) -> controller::State {
        self.state.read().await.clone()
    }
}

#[get("/health")]
pub async fn health(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok().json("healthy")
}

#[get("/")]
pub async fn index(c: WebData<Manager>, _req: HttpRequest) -> impl Responder {
    let state = c.state().await;
    HttpResponse::Ok().json(&state)
}
