mod admin;
mod files;
mod password;
mod ssh_keys;
mod username;

use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{delete, get},
    Json, Router,
};
use http::StatusCode;
use num_traits::cast::ToPrimitive;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::Authentication,
    error,
    file::{FileDb, FileId},
    state::AppState,
};

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .nest("/admin", admin::router())
        .nest("/files", files::router())
        .nest("/password", password::router())
        .nest("/ssh-keys", ssh_keys::router())
        .nest("/username", username::router())
        .route("/", get(handle_get))
        .route("/", delete(handle_delete))
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

async fn handle_delete(
    auth: Authentication,
    State(db): State<PgPool>,
    State(file_db): State<Arc<FileDb>>,
    Path(user_id): Path<Uuid>,
) -> error::Result<Response> {
    if !(auth.is_admin || auth.user_id == user_id) {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    let mut tx = db.begin().await?;

    let files = sqlx::query!(
        r#"delete from files where uploader_id=$1 returning file_id as "file_id: FileId""#,
        user_id,
    )
    .fetch_all(&mut *tx)
    .await?;

    for file in files {
        _ = file_db.file_store().remove(file.file_id).await;
    }

    sqlx::query!(
        r#"
            delete from users where user_id=$1
        "#,
        user_id,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(().into_response())
}
