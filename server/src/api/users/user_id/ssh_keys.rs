mod fingerprint;

use anyhow::Context;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{
    serde::{ts_milliseconds, ts_milliseconds_option},
    DateTime, Utc,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use ssh_key::PublicKey;
use uuid::Uuid;

use crate::{auth::Authentication, error, state::AppState};

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .nest("/:fingerprint", fingerprint::router())
        .route("/", get(handle_get))
        .route("/", post(handle_post))
}

#[derive(Serialize, Debug)]
struct PublicKeyInfo {
    name: String,
    fingerprint: String,
    #[serde(with = "ts_milliseconds")]
    add_date: DateTime<Utc>,
    #[serde(with = "ts_milliseconds_option")]
    last_use: Option<DateTime<Utc>>,
}

async fn handle_get(
    auth: Authentication,
    State(db): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> error::Result<Response> {
    if auth.user_id != user_id {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    let keys = sqlx::query!(
        r#"
            select name, public_key_fingerprint, add_date, last_use
                from ssh_keys
                where user_id=$1
        "#,
        user_id,
    )
    .map(|row| PublicKeyInfo {
        name: row.name,
        fingerprint: row.public_key_fingerprint,
        add_date: row.add_date,
        last_use: row.last_use,
    })
    .fetch_all(&db)
    .await?;

    Ok(Json(keys).into_response())
}

#[derive(Debug, Deserialize)]
struct PostData {
    name: String,
    public_key: PublicKey,
}

async fn handle_post(
    auth: Authentication,
    State(db): State<PgPool>,
    Path(user_id): Path<Uuid>,
    Json(data): Json<PostData>,
) -> error::Result<Response> {
    if auth.user_id != user_id {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    let fingerprint = data
        .public_key
        .fingerprint(ssh_key::HashAlg::Sha512)
        .to_string();

    let mut tx = db.begin().await?;

    let exists = sqlx::query!(
        r#"
            select
                exists(select * from users where user_id=$1)
                    as user_exists,
                exists(select * from ssh_keys where user_id=$1 and public_key_fingerprint=$2)
                    as key_exists
        "#,
        user_id,
        fingerprint,
    )
    .fetch_one(&mut *tx)
    .await?;

    if !exists.user_exists.unwrap_or(false) {
        tx.commit().await?;
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    if exists.key_exists.unwrap_or(false) {
        tx.commit().await?;
        return Ok(StatusCode::CONFLICT.into_response());
    }

    sqlx::query!(
        r#"
            insert into ssh_keys (user_id, public_key_fingerprint, public_key, name)
                values ($1, $2, $3, $4)
        "#,
        user_id,
        fingerprint,
        data.public_key
            .to_openssh()
            .context("could not encode public key")?,
        data.name,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(().into_response())
}
