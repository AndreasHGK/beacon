use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{auth::Authentication, error, state::AppState};

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/", get(handle_get))
}

async fn handle_get(
    auth: Authentication,
    State(db): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> error::Result<Response> {
    if !(auth.is_admin || user_id == auth.user_id) {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    let row = sqlx::query!("select is_admin from users where user_id=$1", user_id)
        .fetch_optional(&db)
        .await?;
    let Some(row) = row else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Json(row.is_admin).into_response())
}
