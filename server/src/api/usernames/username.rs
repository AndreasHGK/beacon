use std::collections::HashMap;

use anyhow::Context;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use http::StatusCode;
use sqlx::PgPool;

use crate::{error, state::AppState};

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/", get(handle_get))
}

async fn handle_get(
    State(db): State<PgPool>,
    Path(username): Path<String>,
) -> error::Result<Response> {
    let row = sqlx::query!("select user_id from users where username = $1", username)
        .fetch_optional(&db)
        .await
        .context("could not fetch user")?;

    let Some(row) = row else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let mut resp = HashMap::new();
    resp.insert("user_id", row.user_id);
    Ok(Json(resp).into_response())
}
