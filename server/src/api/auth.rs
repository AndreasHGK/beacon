mod password;
mod ssh;

use axum::Router;

use crate::state::AppState;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .nest("/password", password::router())
        .nest("/ssh", ssh::router())
}
