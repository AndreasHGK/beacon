mod user_id;

use std::sync::Arc;

use anyhow::Context as _;
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use http::StatusCode;
use num_traits::ToPrimitive;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::{
    auth::Authentication,
    config::Config,
    error,
    session::{create_session, store_session},
    state::AppState,
};

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .nest("/:user_id", user_id::router())
        .route("/", get(handle_get))
        .route("/", post(handle_post))
}

#[derive(Serialize)]
struct UserData {
    id: Uuid,
    username: String,
    total_storage_space: u64,
    created_at: DateTime<Utc>,
    is_admin: bool,
}

async fn handle_get(auth: Authentication, State(db): State<PgPool>) -> error::Result<Response> {
    if !auth.is_admin {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    let rows = sqlx::query!(
        r#"
            select user_id, username, is_admin, created_at, sum(files.file_size) as "total_size"
                from users
                    left outer join files on users.user_id=files.uploader_id
                group by users.user_id
        "#,
    )
    .map(|row| UserData {
        id: row.user_id,
        username: row.username,
        total_storage_space: row.total_size.and_then(|v| v.to_u64()).unwrap_or(0),
        created_at: row.created_at,
        is_admin: row.is_admin,
    })
    .fetch_all(&db)
    .await
    .context("could not fetch users")?;

    Ok(Json(rows).into_response())
}

#[derive(Deserialize)]
struct CreateUser {
    username: String,
    password: String,
}

async fn handle_post(
    cookies: Cookies,
    State(db): State<PgPool>,
    State(config): State<Arc<Config>>,
    Json(request): Json<CreateUser>,
) -> error::Result<Response> {
    if !config.public_config.allow_registering {
        return Ok((StatusCode::FORBIDDEN, "User registering has been disabled").into_response());
    }

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
