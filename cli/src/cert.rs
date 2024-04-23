use std::path::Path;

use anyhow::{anyhow, Context};
use rcgen::{Certificate, CertificateParams, KeyPair};
use rsa::pkcs8::EncodePrivateKey;
use ssh_key::LineEnding;
use time::{Duration, OffsetDateTime};

/// Generate a new x509 certificate using the local private ssh key.
pub fn generate_certificate(
    name: &str,
    valid_for: Duration,
) -> anyhow::Result<(Certificate, KeyPair)> {
    // Define the path of the user's private ssh key as `$HOME/.ssh/id_rsa`.
    let ssh_path = home::home_dir()
        .context("home directory could not be found")?
        .join(Path::new(".ssh"));
    let priv_key_path = ssh_path.join("id_rsa");

    // Try to read the private key.
    let priv_key = ssh_key::PrivateKey::read_openssh_file(&priv_key_path)
        .context("error while trying to read private ssh key")?;
    // Encode the private key using the PEM format so it can be read by rcgen.
    let priv_key_pem = match priv_key.key_data() {
        ssh_key::private::KeypairData::Rsa(key) => {
            let key: rsa::RsaPrivateKey = key
                .try_into()
                .context("could not convert RSA private key")?;
            key.to_pkcs8_pem(LineEnding::CR)
                .context("could not encode RSA private key as PEM")?
        }
        _ => return Err(anyhow!("unsupported ssh key algorithm")),
    };
    let key_pair = KeyPair::from_pem(&priv_key_pem).context("could not read keypair")?;

    // Generate a self-signed certificate and return it.
    let cert = CertificateParams::new(vec![name.to_string()])
        .and_then(|mut v| {
            let now = OffsetDateTime::now_utc();
            v.not_before = now;
            v.not_after = now + valid_for;
            v.self_signed(&key_pair)
        })
        .context("failed to generate client certificate")?;
    Ok((cert, key_pair))
}
