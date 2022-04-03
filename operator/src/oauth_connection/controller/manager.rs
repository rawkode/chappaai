use crate::kubernetes::controller;
use crate::oauth_connection::OAuthConnection;
use futures::{future::BoxFuture, FutureExt, StreamExt};
use kube::runtime::reflector::Store;
use kube::{
    api::{Api, ListParams},
    client::Client,
    runtime::controller::{Context, Controller},
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Manager {
    state: Arc<RwLock<controller::State>>,
}

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

        let oauth_connections = Api::<OAuthConnection>::all(client);

        // Ensure CRD is installed before loop-watching
        let _ = oauth_connections
            .list(&ListParams::default().limit(1))
            .await
            .expect("is the crd installed? please run: cargo run --bin crdgen | kubectl apply -f -");

        // All good. Start controller and return its future.
        let controller = Controller::new(oauth_connections, ListParams::default());

        let store = controller.store();

        let controller = controller
            .run(
                super::reconciler::reconcile,
                super::reconciler::error_policy,
                context,
            )
            .filter_map(|x| async move { std::result::Result::ok(x) })
            .for_each(|_| futures::future::ready(()))
            .boxed();

        (Self { state }, store, controller)
    }

    /// State getter
    pub async fn state(&self) -> controller::State {
        self.state.read().await.clone()
    }
}
