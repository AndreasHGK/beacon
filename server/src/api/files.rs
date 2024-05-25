use std::{env, io, sync::Arc};

use axum::{
    extract::{Request, State},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use futures::TryStreamExt;
use http::StatusCode;
use tokio_util::io::StreamReader;
use tracing::{debug, error, info};

use crate::{auth::Authentication, file::FileDb, state::AppState};

mod file_id;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .nest("/:file_id", file_id::router())
        .route("/", post(handle_post))
}

async fn handle_post(
    auth: Authentication,
    State(file_store): State<Arc<FileDb>>,
    req: Request,
) -> Result<Response, StatusCode> {
    debug!(?auth.user_id, "Started file upload");

    let file_name = req
        .headers()
        .get("file_name")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("file")
        .to_string();

    let file = file_store
        .create(
            auth.user_id,
            file_name,
            StreamReader::new(req.into_body().into_data_stream().map_err(io::Error::other)),
        )
        .await
        .map_err(|err| {
            error!("Could not store file: {err:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!(?auth.user_id, ?file.file_name, ?file.file_size, "A file was uploaded");
    Ok(format!(
        "{}/files/{}/{}",
        env::var("EXTERNAL_URL").unwrap(), // todo: store in config
        file.file_id,
        file.file_name,
    )
    .into_response())
}
