use std::{env, io, sync::Arc};

use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use beacon_panel_shared::file::FileId;
use futures::TryStreamExt;
use tokio_util::io::StreamReader;

use crate::file_store::FileStore;

pub async fn handle_upload(
    State(file_store): State<Arc<FileStore>>,
    req: Request,
) -> Result<Response, StatusCode> {
    log::info!("Received new file!");

    let id = FileId::random();
    file_store
        .put(
            id,
            StreamReader::new(req.into_body().into_data_stream().map_err(io::Error::other)),
        )
        .await
        .map_err(|err| {
            log::error!("Could not store file: {err:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(format!(
        "file {}/files/{id}/file",
        env::var("EXTERNAL_URL").unwrap() // todo: store in config
    )
    .into_response())
}
