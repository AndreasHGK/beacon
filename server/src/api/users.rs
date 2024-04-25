use anyhow::Context as _;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error;

#[derive(Serialize)]
pub struct UserData {
    pub username: String,
}

pub async fn get_user(
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
