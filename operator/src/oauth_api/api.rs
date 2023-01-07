use std::sync::Arc;

use crate::{ApplicationState, Result};

use axum::{Extension, Json};

pub async fn list(Extension(state): Extension<Arc<ApplicationState>>) -> Result<Json<Vec<String>>> {
    let oauth_apis = &state.oauth_apis.state();

    let oauth_api_names: Vec<String> = oauth_apis
        .iter()
        .map(|service| {
            let meta = &service.metadata;
            meta.name.clone().unwrap_or_else(|| String::from("Unknown"))
        })
        .collect();

    Ok(Json(oauth_api_names))
}
