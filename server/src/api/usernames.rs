mod username;

use axum::Router;

use crate::state::AppState;

pub(super) fn router() -> Router<AppState> {
    Router::new().nest("/:username", username::router())
}
