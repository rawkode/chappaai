use crate::{api_version, Error};
use chrono::prelude::*;
use kube::{
    api::{Patch},
    client::Client,
    runtime::events::{Event, Recorder, Reporter}, ResourceExt,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
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

pub async fn state_change<A, B>(
    resource: &Arc<A>,
    recorder: &Recorder,
    event: Event,
    status: B,
) -> Result<(), Error>
where
    A: ResourceExt + DeserializeOwned,
    B: Serialize,
{
    let _new_status = Patch::Apply(json!({
        "apiVersion": api_version(),
        "kind": "OAuthConnection",
        "status": status,
    }));

    let _name = resource.name();

    // let patch_params = PatchParams::apply("chappaai").force();
    // let _ = client
    //     .patch_status(&name, &patch_params, &new_status)
    //     .await
    //     .map_err(Error::KubeError)?;

    recorder.publish(event).await.map_err(Error::KubeError)?;

    Ok(())
}
