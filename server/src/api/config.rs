use std::sync::Arc;

use axum::{extract::State, routing::get, Json, Router};

use crate::{
    config::{Config, PublicConfig},
    state::AppState,
};

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/", get(handle_get))
}

async fn handle_get(State(config): State<Arc<Config>>) -> Json<PublicConfig> {
    Json(config.public_config.clone())
}
