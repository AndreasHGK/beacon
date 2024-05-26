use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{get, put},
    Json, Router,
};
use http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{auth::Authentication, error, state::AppState};

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handle_get))
        .route("/", put(handle_put))
}

/// Get the username corresponding with the user id.
async fn handle_get(
    auth: Authentication,
    State(db): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> error::Result<Response> {
    if !(auth.user_id == user_id || auth.is_admin) {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    let optional_row = sqlx::query!("select username from users where user_id=$1", user_id)
        .fetch_optional(&db)
        .await?;

    let Some(row) = optional_row else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Json(row.username).into_response())
}

/// Change the user's username.
async fn handle_put(
    auth: Authentication,
    State(db): State<PgPool>,
    Path(user_id): Path<Uuid>,
    Json(username): Json<String>,
) -> error::Result<Response> {
    if !(auth.user_id == user_id || auth.is_admin) {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    let mut tx = db.begin().await?;

    let exists = sqlx::query!(
        "select exists(select * from users where username=$1)",
        username,
    )
    .fetch_one(&mut *tx)
    .await?
    .exists
    .unwrap_or(false);

    if exists {
        tx.commit().await?;
        return Ok(StatusCode::CONFLICT.into_response());
    }

    let row = sqlx::query!(
        r#"update users set username=$1 where user_id=$2 returning user_id"#,
        username,
        user_id,
    )
    .fetch_optional(&db)
    .await?;

    if row.is_none() {
        tx.commit().await?;
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    tx.commit().await?;
    Ok(().into_response())
}
