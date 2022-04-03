use kube::CustomResource;
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
pub struct OAuthConnectionKind {
    api: String,
    scopes: Vec<String>,
    client_id: String,
    client_secret: SecretRef,
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
    Disconnected,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct OAuthConnectionStatus {
    pub phase: Option<OAuthConnectionPhase>,
}
