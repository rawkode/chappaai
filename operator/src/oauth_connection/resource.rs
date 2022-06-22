use crate::kubernetes::get_string_value;
use crate::Error;
use k8s_openapi::api::core::v1::Secret;
use kube::{Api, CustomResource};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    pub api: String,
    pub scopes: Vec<String>,
    pub credentials: CredentialOptions,
}

impl OAuthConnection {
    pub async fn load_client_keys(&self, secrets: Api<Secret>) -> Result<(String, String), Error> {
        match &self.spec.credentials {
            CredentialOptions::SecretRef(secret_ref) => {
                let secret = secrets.get(&secret_ref.name).await.map_err(Error::KubeError)?;

                let client_id = match get_string_value(&secret, &secret_ref.id_key) {
                    Ok(value) => value,
                    Err(error) => return Err(error),
                };

                let client_secret = match get_string_value(&secret, &secret_ref.secret_key) {
                    Ok(value) => value,
                    Err(error) => return Err(error),
                };

                Ok((client_id, client_secret))
            }
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum CredentialOptions {
    SecretRef(SecretRef),
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecretRef {
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

impl From<&OAuthConnectionPhase> for String {
    fn from(phase: &OAuthConnectionPhase) -> Self {
        match phase {
            OAuthConnectionPhase::Initializing => "Initializing".to_string(),
            OAuthConnectionPhase::Disconnected => "Disconnected".to_string(),
        }
    }
}
