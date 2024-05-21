mod files;
mod ssh_keys;

use anyhow::Context;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use http::StatusCode;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{error, state::AppState};

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .nest("/files", files::router())
        .nest("/ssh-keys", ssh_keys::router())
        .route("/", get(handle_get))
}

#[derive(Serialize)]
struct UserData {
    username: String,
}

async fn handle_get(
    State(db): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> error::Result<Response> {
    let row = sqlx::query!("select username from users where user_id = $1", user_id)
        .fetch_optional(&db)
        .await
        .context("could not fetch user")?;

    let Some(row) = row else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Json(UserData {
        username: row.username,
    })
    .into_response())
}
