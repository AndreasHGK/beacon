use axum::Router;

use crate::state::AppState;

mod auth;
mod files;
mod usernames;
mod users;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/files", files::router())
        .nest("/usernames", usernames::router())
        .nest("/users", users::router())
}
