use anyhow::Context;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::Authentication,
    error,
    file::{FileId, FileInfo},
    state::AppState,
};

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/", get(handle_get))
}

async fn handle_get(
    auth: Authentication,
    State(db): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> error::Result<Response> {
    if auth.user_id != user_id {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    let files = sqlx::query!(
        r#"select file_id as "file_id: FileId", file_name, file_size, upload_date from files where uploader_id = $1"#,
        user_id,
    )
    .try_map(|row| {
        Ok(FileInfo {
            file_id: row.file_id,
            file_name: row.file_name,
            file_size: row
                .file_size
                .try_into()
                .context("invalid file size")
                .map_err(|err| sqlx::Error::Decode(err.into()))?,
            upload_date: row.upload_date,
        })
    })
    .fetch_all(&db)
    .await?;

    Ok(Json(files).into_response())
}
