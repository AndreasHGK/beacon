use axum::response::{IntoResponse, Response};
use http::StatusCode;
use tracing::error;

pub type Result<T> = core::result::Result<T, ServerError>;

/// Any internal error that may occur on the server.
pub struct ServerError(anyhow::Error);

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        error!("An internal error occurred: {:?}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "An internal error occurred".to_string(),
        )
            .into_response()
    }
}

impl<E> From<E> for ServerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
