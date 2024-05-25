use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use http::StatusCode;
use rsa::{Pkcs1v15Encrypt, RsaPublicKey};
use serde::Deserialize;
use sqlx::PgPool;
use ssh_key::{Fingerprint, PublicKey};

use crate::{auth::ssh::SSHAuthState, error, state::AppState};

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/", post(handle_post))
}

#[derive(Deserialize)]
struct PostData {
    username: String,
    fingerprint: Fingerprint,
}

async fn handle_post(
    State(db): State<PgPool>,
    State(ssh): State<Arc<SSHAuthState>>,
    Json(data): Json<PostData>,
) -> error::Result<Response> {
    let Some(row) = sqlx::query!(
        r#"
            select public_key, users.user_id
                from ssh_keys join users on ssh_keys.user_id=users.user_id
                where users.username=$1 and public_key_fingerprint=$2
        "#,
        data.username,
        data.fingerprint.to_string(),
    )
    .fetch_optional(&db)
    .await?
    else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let public_key =
        PublicKey::from_openssh(&row.public_key).context("could not parse public key")?;

    let ticket = ssh.new_ticket(row.user_id, data.fingerprint).await;
    let serialized_ticket = serde_json::to_string(&ticket)?;

    // TODO: support Ed25519
    let encrypted = match public_key.key_data() {
        ssh_key::public::KeyData::Rsa(rsa) => {
            let rsa_pubkey: RsaPublicKey =
                rsa.try_into().context("failed to convert RSA public key")?;

            rsa_pubkey.encrypt(
                &mut rand::thread_rng(),
                Pkcs1v15Encrypt,
                serialized_ticket.as_bytes(),
            )?
        }
        _ => {
            return Ok((
                StatusCode::UNPROCESSABLE_ENTITY,
                "SSH key algorithm not supported",
            )
                .into_response())
        }
    };

    Ok(encrypted.into_response())
}
