mod step1;
mod step2;

use axum::Router;

use crate::state::AppState;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .nest("/step1", step1::router())
        .nest("/step2", step2::router())
}
