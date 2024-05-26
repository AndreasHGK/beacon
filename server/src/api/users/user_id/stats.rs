use axum::Router;

use crate::{auth::Authentication, state::AppState};

pub(super) fn router() -> Router<AppState> {
    Router::new()
}

async fn handle_get(auth: Authentication, )
