use axum::Router;

use crate::state::AppState;

mod auth;
mod config;
mod files;
mod usernames;
mod users;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/config", config::router())
        .nest("/files", files::router())
        .nest("/usernames", usernames::router())
        .nest("/users", users::router())
}
