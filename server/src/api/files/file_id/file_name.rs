mod content;

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get},
    Json, Router,
};
use tracing::{error, info};

use crate::state::AppState;
use crate::{auth::Authentication, file::FileId, FileDb};

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .nest("/content", content::router())
        .route("/", get(handle_get))
        .route("/", delete(handle_delete))
}

async fn handle_get(
    State(file_store): State<Arc<FileDb>>,
    Path((file_id, file_name)): Path<(FileId, String)>,
) -> Response {
    let file_info = match file_store.file_info(file_id).await {
        Err(err) => {
            error!("Could not get file info from file store: {err}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        Ok(None) => {
            return StatusCode::NOT_FOUND.into_response();
        }
        Ok(Some(v)) if v.file_name != file_name => {
            return StatusCode::NOT_FOUND.into_response();
        }
        Ok(Some(v)) => v,
    };

    Json(file_info).into_response()
}

async fn handle_delete(
    auth: Authentication,
    State(file_store): State<Arc<FileDb>>,
    Path((file_id, file_name)): Path<(FileId, String)>,
) -> crate::error::Result<Response> {
    let mut tx = file_store.db().begin().await?;

    let Some(uploader_id) = sqlx::query!(
        r#"
            select uploader_id
                from files
                where file_id=$1 and file_name=$2
        "#,
        file_id as FileId,
        file_name
    )
    .fetch_optional(&mut *tx)
    .await?
    .map(|row| row.uploader_id) else {
        // The file was not found, return.
        tx.commit().await?;
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    // The user should only be able to delete a file if they're either the original uploader of the
    // file or a site-wide admin.
    if uploader_id != auth.user_id && !auth.is_admin {
        tx.commit().await?;
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    info!(?file_id, "Deleting file.");

    sqlx::query!("delete from files where file_id=$1", file_id as FileId)
        .execute(&mut *tx)
        .await?;

    file_store.file_store().remove(file_id).await?;
    tx.commit().await?;

    Ok(().into_response())
}
