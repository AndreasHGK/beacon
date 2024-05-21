use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::get,
    Router,
};
use http::{HeaderName, StatusCode};
use tokio_util::io::ReaderStream;
use tracing::error;

use crate::{
    file::{FileDb, FileId},
    state::AppState,
};

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/", get(handle_get))
}

async fn handle_get(
    State(file_store): State<Arc<FileDb>>,
    Path((file_id, file_name)): Path<(FileId, String)>,
) -> Response {
    let resp_stream = match file_store.content(file_id, &file_name).await {
        Ok(Some(v)) => v,
        Ok(None) => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap();
        }
        Err(err) => {
            error!("Could not get file from file store: {err}");
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap();
        }
    };

    Response::builder()
        .header(
            HeaderName::from_static("content-disposition"),
            format!("attachment; filename=\"{file_name}\""),
        )
        .header(
            HeaderName::from_static("content-type"),
            "application/octet-stream",
        )
        .body(Body::from_stream(ReaderStream::new(resp_stream)))
        .unwrap()
}
