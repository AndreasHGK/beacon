use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::delete,
    Router,
};
use http::StatusCode;
use sqlx::PgPool;
use ssh_key::Fingerprint;
use uuid::Uuid;

use crate::{auth::Authentication, error, state::AppState};

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/", delete(handle_delete))
}

async fn handle_delete(
    auth: Authentication,
    State(db): State<PgPool>,
    Path((user_id, fingerprint)): Path<(Uuid, Fingerprint)>,
) -> error::Result<Response> {
    if auth.user_id != user_id {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    let row = sqlx::query!(
        r#"
            delete from ssh_keys
                where user_id=$1 and public_key_fingerprint=$2
                returning true as found
        "#,
        user_id,
        fingerprint.to_string(),
    )
    .fetch_optional(&db)
    .await?;

    if row.is_none() {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    Ok(().into_response())
}
