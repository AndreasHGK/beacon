use std::{
    fmt::{self, Display},
    str::FromStr,
};

use anyhow::Context;
use chrono::{serde::ts_milliseconds, DateTime, Duration, Utc};
use hex::{FromHex, FromHexError};
use serde::{de::Visitor, Deserialize, Serialize};
use sqlx::{Postgres, Transaction};
use time::OffsetDateTime;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

/// Information about a session.
#[derive(Serialize)]
pub struct SessionInfo {
    /// The unique identifier for the session.
    pub token: SessionToken,
    /// The user to which the session belongs.
    pub user_id: Uuid,
    /// The date and time at which the session will no longer be considered valid.
    #[serde(with = "ts_milliseconds")]
    pub valid_until: DateTime<Utc>,
}

/// An authentication token to allow a user to prove to the backend they are authenticated.
///
/// The token is 256 bits. 128 bits is the minimum recommended by OWASP:
/// <https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html#session-id-length>
pub struct SessionToken([u8; 32]);

impl SessionToken {
    /// Generate a new random token.
    pub fn random() -> Self {
        // The default random number generator is cryptographically secure:
        // https://docs.rs/rand/latest/rand/rngs/struct.StdRng.html
        Self(rand::random())
    }

    /// Returns the raw bytes of the token.
    pub fn raw(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Creates a new session and inserts it into the database.
pub async fn create_session(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    valid_for: Duration,
) -> anyhow::Result<SessionInfo> {
    let token = loop {
        let token = SessionToken::random();

        let exists = sqlx::query!(
            "select exists(select * from sessions where token = $1)",
            token.raw(),
        )
        .fetch_one(&mut **tx)
        .await
        .context("failed to check if token exists")?
        .exists
        .unwrap_or(false);

        if !exists {
            break token;
        }
    };

    let issued_at = Utc::now();
    let expires_on = issued_at + valid_for;
    sqlx::query!(
        r#"
        insert into sessions (token, user_id, issued_at, expires_on)
            values ($1, $2, $3, $4)
        "#,
        token.raw(),
        user_id,
        issued_at,
        expires_on,
    )
    .execute(&mut **tx)
    .await
    .context("failed to create session")?;

    Ok(SessionInfo {
        user_id,
        valid_until: expires_on,
        token,
    })
}

/// Store a session in the cookies when sending a response.
pub fn store_session(cookies: &Cookies, session: &SessionInfo) -> anyhow::Result<()> {
    let cookie_expire = OffsetDateTime::from_unix_timestamp(session.valid_until.timestamp())
        .context("could not convert date")?;

    let mut cookie = Cookie::new("session-token", session.token.to_string());
    cookie.set_secure(Some(true));
    cookie.set_http_only(Some(true));
    cookie.set_expires(cookie_expire);
    cookie.set_path("/");
    cookie.set_same_site(None);
    cookies.add(cookie);
    let mut cookie = Cookie::new("session-uuid", session.user_id.to_string());
    cookie.set_expires(cookie_expire);
    cookie.set_path("/");
    cookie.set_same_site(None);
    cookies.add(cookie);
    Ok(())
}

impl Display for SessionToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl FromStr for SessionToken {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <[u8; 32]>::from_hex(s).map(SessionToken)
    }
}

impl<'de> Deserialize<'de> for SessionToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = SessionToken;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a hex-encoded FileId")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                SessionToken::from_str(v).map_err(|_| {
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(v),
                        &"a 64-bit hex-encoded integer",
                    )
                })
            }
        }
        deserializer.deserialize_str(V)
    }
}

impl Serialize for SessionToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{self}"))
    }
}
