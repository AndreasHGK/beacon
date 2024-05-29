use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use chrono::{Duration, Utc};
use http::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;

use crate::{auth::Authentication, error, state::AppState};

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/", post(handle_post))
}

#[derive(Deserialize)]
struct PostData {
    invite_code: String,
    /// The amount of seconds for which the invite should remain valid.
    valid_for: u32,
    max_uses: u16,
}

/// Create a new invite code.
async fn handle_post(
    auth: Authentication,
    State(db): State<PgPool>,
    Json(data): Json<PostData>,
) -> error::Result<Response> {
    if !auth.is_admin {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    let valid_for = Duration::seconds(data.valid_for as i64);
    if valid_for > Duration::days(7) || valid_for < Duration::zero() {
        return Ok((
            StatusCode::BAD_REQUEST,
            "`valid_for` field must be between 0 and 7 days (in seconds)",
        )
            .into_response());
    }
    let expires_on = Utc::now() + Duration::seconds(data.valid_for as i64);

    let mut tx = db.begin().await?;

    let exists = sqlx::query!(
        r#"select exists(select * from invites where invite=$1) as "exists!: bool""#,
        &data.invite_code,
    )
    .fetch_one(&mut *tx)
    .await?;

    if exists.exists {
        tx.commit().await?;
        return Ok(StatusCode::CONFLICT.into_response());
    }

    sqlx::query!(
        r#"
            insert into invites (invite, max_uses, valid_until, created_by)
                values ($1, $2, $3, $4)
        "#,
        data.invite_code,
        data.max_uses as i32,
        expires_on,
        auth.user_id,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(().into_response())
}
