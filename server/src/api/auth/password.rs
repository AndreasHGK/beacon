use std::sync::Arc;

use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use chrono::Duration;
use http::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use tower_cookies::Cookies;
use tracing::warn;

use crate::{
    auth::UserAuthFailures,
    error,
    session::{create_session, store_session},
    state::AppState,
};

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/", post(handle_post))
}

#[derive(Deserialize)]
struct AuthenticateForm {
    username: String,
    password: String,
}

async fn handle_post(
    cookies: Cookies,
    State(db): State<PgPool>,
    State(failures): State<Arc<UserAuthFailures>>,
    Json(form): Json<AuthenticateForm>,
) -> error::Result<Response> {
    // Get the current time. This is later used to ensure this requests takes a minimum amount of
    // time.
    let start = tokio::time::Instant::now();

    let mut tx = db.begin().await?;
    // First check if the user exists and if the password matches.
    let row = sqlx::query!(
        "select user_id, password_hash from users where username = $1",
        form.username,
    )
    .fetch_optional(&mut *tx)
    .await?;

    let Some(row) = row else {
        warn!(?form.username, "Tried to log in for unknown user");
        // Authentication failed if the user could not be found.
        tx.commit().await?;
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let password = form.password;
    let hash_check = tokio::task::spawn_blocking(move || {
        Argon2::default().verify_password(
            password.as_bytes(),
            &PasswordHash::new(&row.password_hash).context("failed to parse password hash")?,
        )?;
        anyhow::Result::<()>::Ok(())
    })
    .await
    .context("could not join hasher thread")?;

    if let Err(err) = hash_check {
        let mut failures = failures.users.lock().await;

        let entry = failures.entry(row.user_id).or_insert((0, start));
        // After an hour of no failed attempts we can reset the counter.
        if start - entry.1 > tokio::time::Duration::from_secs(60 * 60) {
            entry.0 = 0;
        }

        // Ensure this doesn't overflow.
        entry.0 = entry.0.saturating_add(1);

        let amt_failures = entry.0;
        // Unlock mutex before sleeping.
        drop(failures);
        // Make the request take at least a set amount of time, capped at 2 seconds.
        tokio::time::sleep_until(
            start + tokio::time::Duration::from_millis(100 * amt_failures.min(20) as u64),
        )
        .await;

        tx.commit().await?;
        warn!(
            "Authentication for user `{}` failed: {err:?}",
            form.username
        );
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    // The user exists and the password matches, we can now create a session.
    let session = create_session(&mut tx, row.user_id, Duration::weeks(1))
        .await
        .context("error while creating session")?;
    tx.commit().await?;

    store_session(&cookies, &session)?;

    Ok(Json(session).into_response())
}
