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
    pub auth: Option<AuthSpecs>,
    pub http: HttpApi,
}

impl OAuthApi {
    pub fn get_authorization_url(&self) -> String {
        match &self.spec.auth {
            Some(AuthSpecs::OAuth2(spec)) => {
                format!(
                    "{}/{}?{}",
                    self.spec.http.base_url,
                    spec.authorization_url,
                    spec.get_authorization_params()
                )
            }
            _ => todo!(),
        }
    }

    pub fn get_token_url(&self) -> String {
        match &self.spec.auth {
            Some(AuthSpecs::OAuth2(spec)) => format!("{}/{}", self.spec.http.base_url, spec.token_url),
            _ => todo!(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HttpApi {
    pub base_url: String,
    pub authorization_header_prefix: Option<String>,

    #[serde(default)]
    pub headers: Vec<HttpHeaders>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HttpHeaders {
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
    pub authorization_url: String,
    #[serde(default)]
    pub authorization_params: Vec<AuthorizationParams>,

    pub refresh_url: Option<String>,

    pub token_url: String,
    pub token_params: Option<TokenParams>,
}

impl OAuth2Spec {
    pub fn get_authorization_params(&self) -> String {
        self.authorization_params.iter().fold(String::from(""), |acc, x| {
            format!("{}&{}={}", acc, x.key, x.value)
        })
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationParams {
    key: String,
    value: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenParams {
    grant_type: String,
}
