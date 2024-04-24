use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Duration;
use http::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::warn;

use crate::{error, session::create_session};

#[derive(Deserialize)]
pub struct AuthenticateForm {
    username: String,
    password: String,
}

pub async fn authenticate(
    State(db): State<PgPool>,
    Json(form): Json<AuthenticateForm>,
) -> error::Result<Response> {
    let mut tx = db.begin().await?;
    // First check if the user exists and if the password matches.
    let row = sqlx::query!(
        "select user_id, password_hash from users where username = $1",
        form.username,
    )
    .fetch_optional(&mut *tx)
    .await?;

    let Some(row) = row else {
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

    Ok(Json(session).into_response())
}
