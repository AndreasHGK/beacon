mod password;

use axum::Router;

use crate::state::AppState;

pub(super) fn router() -> Router<AppState> {
    Router::new().nest("/password", password::router())
}
