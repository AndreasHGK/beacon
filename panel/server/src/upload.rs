use std::{env, io, sync::Arc};

use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use beacon_panel_shared::server::file::FileDb;
use futures::TryStreamExt;
use tokio_util::io::StreamReader;

pub async fn handle_upload(
    State(file_store): State<Arc<FileDb>>,
    req: Request,
) -> Result<Response, StatusCode> {
    log::info!("Received new file!");

    let file_name = req
        .headers()
        .get("file_name")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("file")
        .to_string();

    let file = file_store
        .create(
            file_name,
            StreamReader::new(req.into_body().into_data_stream().map_err(io::Error::other)),
        )
        .await
        .map_err(|err| {
            log::error!("Could not store file: {err:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(format!(
        "{}/files/{}/{}",
        env::var("EXTERNAL_URL").unwrap(), // todo: store in config
        file.file_id,
        file.file_name,
    )
    .into_response())
}
