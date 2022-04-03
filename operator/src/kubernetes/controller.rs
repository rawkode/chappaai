use chrono::prelude::*;
use kube::{client::Client, runtime::events::Reporter};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Serialize)]
pub struct State {
    #[serde(deserialize_with = "from_ts")]
    pub last_event: DateTime<Utc>,

    #[serde(skip)]
    pub reporter: Reporter,
}

impl State {
    pub fn new(name: String) -> Self {
        State {
            last_event: Utc::now(),
            reporter: format!("chappai-{}-controller", name).into(),
        }
    }
}

#[derive(Clone)]
pub struct Data {
    pub client: Client,
    pub state: Arc<RwLock<State>>,
}
