pub(crate) use chappaai::oauth_api::OAuthApi;
pub(crate) use chappaai::oauth_connection::OAuthConnection;
pub(crate) use kube::CustomResourceExt;

fn main() {
    print!("{}", serde_yaml::to_string(&OAuthApi::crd()).unwrap());
    print!("{}", serde_yaml::to_string(&OAuthConnection::crd()).unwrap());
}
