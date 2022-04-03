use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    group = "chappaai.dev",
    version = "v1",
    kind = "OAuthApi",
    status = "OAuthApiStatus",
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct OAuthApiKind {
    auth: Option<AuthSpecs>,
    http: HttpApi,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct HttpApi {
    base_url: String,
    authorization_header_prefix: Option<String>,

    #[serde(default)]
    headers: Vec<HttpHeaders>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct HttpHeaders {
    key: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub enum OAuthApiPhase {
    Registered,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct OAuthApiStatus {
    pub phase: Option<OAuthApiPhase>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AuthSpecs {
    OAuth2(OAuth2Spec),
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2Spec {
    authorization_url: String,
    #[serde(default)]
    authorization_params: Vec<AuthorizationParams>,

    refresh_url: Option<String>,

    token_url: String,
    token_params: Option<TokenParams>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AuthorizationParams {
    key: String,
    value: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct TokenParams {
    grant_type: String,
}
