use k8s_openapi::{api::core::v1::Secret, ByteString};
use kube::{Api, CustomResource};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    group = "chappaai.dev",
    version = "v1",
    kind = "OAuthConnection",
    status = "OAuthConnectionStatus",
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct OAuthConnectionSpec {
    api: String,
    scopes: Vec<String>,
    client_id: String,
    client_secret: SecretRef,
}

impl OAuthConnectionSpec {
    pub fn get_client_id(self: &Self) -> &String {
        &self.client_id
    }

    pub fn get_client_secret(self: &Self) -> (&String, &String) {
        (&self.client_secret.name, &self.client_secret.key)
    }

    pub async fn load_client_secret(self: &Self, secrets: Api<Secret>) -> Result<ByteString, Error> {
        let (secret_name, secret_key) = self.get_client_secret();

        let secret = secrets.get(&secret_name).await.map_err(Error::KubeError)?;

        if secret.data.is_none() {
            return Err(Error::GenericError(
                "Secret has no data to contain client secret".into(),
            ));
        }

        let secret = secret.data.as_ref().unwrap();

        let client_secret = match secret.get(secret_key) {
            Some(client_secret) => client_secret,
            None => {
                return Err(Error::GenericError(
                    "Secret has no data to contain client secret".into(),
                ))
            }
        };

        Ok(client_secret.clone())
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct SecretRef {
    namespace: Option<String>,
    name: String,
    key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub enum OAuthConnectionPhase {
    Initializing,
    Disconnected,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct OAuthConnectionStatus {
    pub phase: Option<OAuthConnectionPhase>,
}
