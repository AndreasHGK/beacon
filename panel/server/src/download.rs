use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderName, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use beacon_panel_shared::{file::FileId, server::file::FileDb};
use log::error;
use tokio_util::io::ReaderStream;

pub async fn file_info(
    State(file_store): State<Arc<FileDb>>,
    Path((file_id, file_name)): Path<(FileId, String)>,
) -> Response {
    let file = match file_store.file_info(file_id).await {
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

    if file.file_name != file_name {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap();
    }

    Json::from(file).into_response()
}

pub async fn file_content(
    State(file_store): State<Arc<FileDb>>,
    Path((file_id, file_name)): Path<(FileId, String)>,
) -> Response {
    // todo: check if the name matches the name stored here

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
