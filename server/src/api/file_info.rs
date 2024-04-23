use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use tracing::error;

use crate::{file::FileId, FileDb};

pub async fn file_info(
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
