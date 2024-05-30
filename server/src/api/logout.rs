use anyhow::anyhow;
use axum::{extract::State, routing::post, Router};
use sqlx::PgPool;
use tower_cookies::Cookies;

use crate::{auth::Authentication, error, session::remove_session, state::AppState};

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/", post(handle_post))
}

/// Logs out the user, removing their current session.
async fn handle_post(
    auth: Authentication,
    cookies: Cookies,
    State(db): State<PgPool>,
) -> error::Result<()> {
    remove_session(&cookies);

    let result = sqlx::query!(
        r#"
            delete from sessions where token = $1
        "#,
        auth.token.raw(),
    )
    .execute(&db)
    .await?;

    if result.rows_affected() != 1 {
        return Err(anyhow!("Failed to delete session").into());
    }
    Ok(())
}
