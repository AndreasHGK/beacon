use std::sync::Arc;

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use chrono::Duration;
use http::StatusCode;
use sqlx::PgPool;
use tower_cookies::Cookies;
use tracing::warn;

use crate::{
    auth::ssh::{SSHAuthState, Ticket},
    error,
    session::{self, store_session},
    state::AppState,
};

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/", post(handle_post))
}

async fn handle_post(
    cookies: Cookies,
    State(db): State<PgPool>,
    State(ssh): State<Arc<SSHAuthState>>,
    Json(ticket): Json<Ticket>,
) -> error::Result<Response> {
    let Some((user, fingerprint)) = ssh.validate_response_ticket(ticket).await else {
        warn!("User supplied an unknown ticket");
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let mut tx = db.begin().await?;

    // Check if the user still exists (user_id is a foreign key) and the ssh key is still added.
    let row = sqlx::query!(
        "select exists(select * from ssh_keys where user_id=$1 and public_key_fingerprint=$2)",
        user,
        fingerprint.to_string(),
    )
    .fetch_one(&mut *tx)
    .await?;
    // Reject the user if the above is not true.
    if !row.exists.unwrap_or(false) {
        tx.commit().await?;
        warn!("User or user SSH key has been removed since ssh auth step 1");
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    let session = session::create_session(&mut tx, user, Duration::minutes(1)).await?;
    tx.commit().await?;
    store_session(&cookies, &session)?;

    Ok(Json(session).into_response())
}
