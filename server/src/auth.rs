use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use axum_extra::extract::CookieJar;
use sqlx::PgPool;
use tracing::{debug, error, trace, warn};
use uuid::Uuid;

use crate::session::SessionToken;

/// A HTTP extractor that checks if the user is authorized and provides the user info if this is
/// the case. Returns an UNAUTHORIZED response otherwise.
#[derive(Debug)]
pub struct Authentication {
    /// The unique identifier of the authenticated user.
    pub user_id: Uuid,
    /// The username of the authenticated user.
    pub username: String,
    /// True if the user has admininistrator permissions.
    pub is_admin: bool,
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Authentication
where
    PgPool: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the session token cookie and parse it.
        let cookies = CookieJar::from_headers(&parts.headers);
        let token = cookies
            .get("session-token")
            .ok_or_else(|| {
                trace!("Missing `session-token` cookie");
                StatusCode::UNAUTHORIZED
            })?
            .value_trimmed()
            .parse::<SessionToken>()
            .map_err(|err| {
                debug!("Unable to parse `session-token` cookie: {err}");
                StatusCode::UNAUTHORIZED
            })?;

        // Check if a session with the provided token exists. If not, return an unauthorized
        // response.
        let db = PgPool::from_ref(state);
        let row = sqlx::query!(
            r#"
                select users.user_id, username, is_admin
                    from sessions join users on sessions.user_id = users.user_id
                    where issued_at < now() and expires_on > now() and token = $1 
            "#,
            token.raw()
        )
        .fetch_optional(&db)
        .await
        .map_err(|err| {
            error!("Error fetching session: {err}");
            StatusCode::UNAUTHORIZED
        })?
        .ok_or_else(|| {
            warn!("Unknown session token used");
            StatusCode::UNAUTHORIZED
        })?;

        Ok(Authentication {
            user_id: row.user_id,
            username: row.username,
            is_admin: row.is_admin,
        })
    }
}
