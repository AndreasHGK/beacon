use anyhow::Context;
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::put,
    Json, Router,
};
use http::StatusCode;
use rand::rngs::OsRng;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::warn;
use uuid::Uuid;

use crate::{auth::Authentication, error, state::AppState};

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/", put(handle_put))
}

#[derive(Deserialize)]
struct PutData {
    /// The current password of the user making the request.
    sender_current_password: String,
    /// The desired new password for the target user.
    target_new_password: String,
}

/// Update a user's password.
async fn handle_put(
    auth: Authentication,
    State(db): State<PgPool>,
    Path(user_id): Path<Uuid>,
    Json(data): Json<PutData>,
) -> error::Result<Response> {
    if !(auth.is_admin || user_id == auth.user_id) {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    let mut tx = db.begin().await?;

    // Check if the target user exists.
    if !sqlx::query!(
        "select exists(select * from users where user_id=$1)",
        user_id
    )
    .fetch_one(&mut *tx)
    .await?
    .exists
    .unwrap_or(false)
    {
        tx.commit().await?;
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    // Check if the sender's password matches.
    {
        let Some(row) = sqlx::query!(
            "select password_hash, username from users where user_id=$1",
            user_id
        )
        .fetch_optional(&mut *tx)
        .await?
        else {
            return Ok(StatusCode::NOT_FOUND.into_response());
        };

        let hash_check = tokio::task::spawn_blocking(move || {
            Argon2::default().verify_password(
                data.sender_current_password.as_bytes(),
                &PasswordHash::new(&row.password_hash).context("failed to parse password hash")?,
            )?;
            anyhow::Result::<()>::Ok(())
        })
        .await
        .context("could not join hasher thread")?;

        if let Err(err) = hash_check {
            tx.commit().await?;
            warn!("Authentication for user `{}` failed: {err:?}", row.username);

            return Ok(StatusCode::UNAUTHORIZED.into_response());
        }
    }

    let password = data.target_new_password;
    let password_hash = tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
    })
    .await
    .context("could not join hasher thread")?
    .context("failed to hash password")?;

    sqlx::query!(
        r#"
            update users
                set password_hash=$2
                where user_id=$1
        "#,
        user_id,
        password_hash,
    )
    .execute(&mut *tx)
    .await?;

    // Remove all active sessions for the user, forcing them to log in again.
    sqlx::query!("delete from sessions where user_id=$1", user_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(().into_response())
}
