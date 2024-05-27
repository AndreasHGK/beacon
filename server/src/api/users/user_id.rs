mod files;
mod password;
mod ssh_keys;
mod username;

use anyhow::Context;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use http::StatusCode;
use num_traits::cast::ToPrimitive;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{error, state::AppState};

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .nest("/files", files::router())
        .nest("/password", password::router())
        .nest("/ssh-keys", ssh_keys::router())
        .nest("/username", username::router())
        .route("/", get(handle_get))
}

#[derive(Serialize)]
struct UserData {
    username: String,
    total_storage_space: u64,
}

async fn handle_get(
    State(db): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> error::Result<Response> {
    let row = sqlx::query!(
        r#"
            select username, sum(files.file_size) as "total_size"
                from users
                    left outer join files on users.user_id=files.uploader_id
                where user_id = $1
                group by users.user_id
        "#,
        user_id,
    )
    .fetch_optional(&db)
    .await
    .context("could not fetch user")?;

    let Some(row) = row else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Json(UserData {
        username: row.username,
        total_storage_space: row
            .total_size
            .map(|v| v.to_u64().context("could not convert file size"))
            .transpose()?
            .unwrap_or(0),
    })
    .into_response())
}
