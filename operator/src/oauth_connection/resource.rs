use k8s_openapi::{api::core::v1::Secret, ByteString};
use kube::{client, Api, CustomResource};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{kubernetes::get_string_value, Error};

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
    credentials: CredentialOptions,
}

impl OAuthConnectionSpec {
    pub async fn load_client_keys(self: &Self, secrets: Api<Secret>) -> Result<(String, String), Error> {
        match &self.credentials {
            CredentialOptions::SecretRef(secret_ref) => {
                let secret = secrets.get(&secret_ref.name).await.map_err(Error::KubeError)?;

                let client_id = match get_string_value(&secret, &secret_ref.id_key) {
                    Ok(value) => value,
                    Err(error) => return Err(error),
                };

                println!("Successfully got the cvlient id {}", client_id);

                let client_secret = match get_string_value(&secret, &secret_ref.secret_key) {
                    Ok(value) => value,
                    Err(error) => return Err(error),
                };

                println!("Successfully got the client secret {}", client_secret);

                Ok((client_id, client_secret))
            }
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum CredentialOptions {
    SecretRef(SecretRef),
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct SecretRef {
    namespace: Option<String>,
    name: String,
    id_key: String,
    secret_key: String,
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
