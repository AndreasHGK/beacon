use rand::thread_rng;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};
use serde::{Deserialize, Serialize};
use ssh_key::{Fingerprint, PrivateKey};
use tokio::fs;

/// Walks through the user's `~/.ssh` folder and returns the first private key it finds.
pub async fn get_private_key() -> Option<PrivateKey> {
    let ssh_dir = homedir::get_my_home().ok()??.join(".ssh/");

    let mut dir = fs::read_dir(ssh_dir).await.ok()?;
    while let Some(entry) = dir.next_entry().await.ok()? {
        if !entry
            .metadata()
            .await
            .ok()
            .map(|v| v.is_file())
            .unwrap_or(false)
        {
            continue;
        }

        let Ok(data) = fs::read(entry.path()).await else {
            continue;
        };
        let Ok(key) = PrivateKey::from_openssh(data) else {
            continue;
        };
        return Some(key);
    }

    None
}

/// Negotiates a new session with the server using the private key specified. The private key is
/// never sent over the network.
///
/// Note: this relies on the client having a cookie store, as the session will be stored in the
/// cookie store on success.
pub async fn create_session(
    client: &mut reqwest::Client,
    host: &str,
    username: &str,
    key: PrivateKey,
) -> anyhow::Result<()> {
    let resp = client
        .post(format!("{host}/api/auth/ssh/step1"))
        .json(&Step1Payload {
            username,
            fingerprint: key.fingerprint(ssh_key::HashAlg::Sha512),
        })
        .send()
        .await?
        .error_for_status()?;

    let resp = resp.bytes().await?;

    let mut ticket: Ticket = match key.key_data() {
        ssh_key::private::KeypairData::Rsa(rsa) => {
            let rsa_key: RsaPrivateKey = rsa.try_into()?;

            let decr = rsa_key.decrypt_blinded(&mut thread_rng(), Pkcs1v15Encrypt, &resp)?;
            serde_json::from_slice(&decr)?
        }
        _ => return Err(anyhow::anyhow!("unsupported SSH key algorirthm")),
    };
    ticket.nonce = ticket.nonce.wrapping_add(1);

    client
        .post(format!("{host}/api/auth/ssh/step2"))
        .json(&ticket)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

#[derive(Serialize)]
struct Step1Payload<'a> {
    username: &'a str,
    fingerprint: Fingerprint,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
struct Ticket {
    nonce: u128,
    timestamp: i64,
    subject: String,
}
