pub mod files;
pub mod ssh_keys;

use std::collections::HashMap;

use anyhow::Context as _;
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use chrono::Duration;
use http::StatusCode;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::{
    error,
    session::{create_session, store_session},
};

#[derive(Serialize)]
pub struct UserData {
    pub username: String,
}

pub async fn get_user(
    State(db): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> error::Result<Response> {
    let row = sqlx::query!("select username from users where user_id = $1", user_id)
        .fetch_optional(&db)
        .await
        .context("could not fetch user")?;

    let Some(row) = row else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Json(UserData {
        username: row.username,
    })
    .into_response())
}

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
}

pub async fn create_user(
    cookies: Cookies,
    State(db): State<PgPool>,
    Json(request): Json<CreateUser>,
) -> error::Result<Response> {
    let mut tx = db.begin().await?;

    let row = sqlx::query!(
        "select exists(select * from users where username = $1)",
        request.username,
    )
    .fetch_one(&mut *tx)
    .await
    .context("could not fetch user")?;

    if row.exists.unwrap_or(false) {
        return Ok((StatusCode::BAD_REQUEST, "Username is taken.").into_response());
    }

    // Hash the password in a separate blocking thread as it is an expensive CPU bound
    // operation.
    let password = request.password;
    let password_hash = tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
    })
    .await
    .context("could not join hasher thread")?
    .context("failed to hash password")?;

    let row = sqlx::query!(
        r#"
        insert into users (user_id, username, password_hash) values(gen_random_uuid(), $1, $2)
            returning user_id
        "#,
        request.username,
        password_hash,
    )
    .fetch_one(&mut *tx)
    .await
    .context("failed to create user in database")?;

    // The user has been created, now make a session for the user.
    let session = create_session(&mut tx, row.user_id, Duration::weeks(2))
        .await
        .context("failed ot create session")?;

    tx.commit().await?;

    store_session(&cookies, &session)?;
    Ok(Json(session).into_response())
}

pub async fn get_username(
    State(db): State<PgPool>,
    Path(username): Path<String>,
) -> error::Result<Response> {
    let row = sqlx::query!("select user_id from users where username = $1", username)
        .fetch_optional(&db)
        .await
        .context("could not fetch user")?;

    let Some(row) = row else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let mut resp = HashMap::new();
    resp.insert("user_id", row.user_id);
    Ok(Json(resp).into_response())
}
