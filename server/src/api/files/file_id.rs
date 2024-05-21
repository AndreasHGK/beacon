mod file_name;

use axum::Router;

use crate::state::AppState;

pub(super) fn router() -> Router<AppState> {
    Router::new().nest("/:file_name", file_name::router())
}
