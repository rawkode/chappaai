use super::OAuthConnection;
use crate::Error;
use kube::{
    runtime::{controller::Action, events::Recorder},
    Client,
};
use std::sync::Arc;

pub fn disconnected(_: Client, _: Recorder, _: Arc<OAuthConnection>) -> Result<Action, Error> {
    Ok(Action::await_change())
}
