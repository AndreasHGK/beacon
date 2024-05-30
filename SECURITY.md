# Security

## Authentication

A user can authenticate in two ways: **password** auth and **SSH key** auth.
Both provide the user with a session token on success.

### Password Auth

The user sends a simple json object with their username and password over HTTPS to
`/api/auth/password` (POST request).
The user's associated password hash is then fetched from the database and the password in the
request is verified.
Argon2id is used for password hashing

### SSH Key Auth

All the following messages are exchanged over HTTPS.

Authenticating using an SSH key happens in two steps.
The first one is performed by making a POST request to `/api/auth/ssh/step1`.
The body of this request is JSON encoded containing the username and public key fingerprint of the
user to authenticate as (a user can have multiple ssh keys added to their account).
Upon receiving this first request, the server verifies whether this account has indeed added the
public key with the corresponding fingerprint to their account.
It then responds with the user's UUID, the current timestamp and a random nonce (called a 'ticket'
internally)
encoded as json and encrypted with the user's public key.

In the second step, the user takes the response from the last step and decrypts this with their
private key. This is only possible if the user knows the private key. They then send a POST request
to `/api/auth/ssh/step2` containing the values of the ticket with `nonce + 1` instead of the
original nonce. If the server finds this matches the data it stored from the previous request, the
session is created and a session key is replied back to the user.

## OWASP Top 10

[https://owasp.org/www-project-top-ten/]
The top 10 security risks published by OWASP have been evaluated in this project.

### Broken Access Control

- `server/src/auth.rs` contains the `Authentication` extractor, which can be re-used in requests deny
  unauthenticated users.
- Access control failures are logged (warning) in the logs of the server.
- Directory listing is not enabled on the server.

### Cryptographic Failures

- User passwords are hashed using Argon2id.
- HTTPS can be enforced when using caddy. This is up to the instance owner to ensure when deploying.
- The session token cookie is marked as secure.
- The CLI disallows non HTTPS urls when in release mode.
- The random functions used are safe for cryptographic use.
  The rand library uses a CSPRNG by default:
  [https://rust-random.github.io/book/guide-rngs.html#cryptographically-secure-pseudo-random-number-generators-csprngs]

### Injection

- SQLX prevents SQL injection by using parameters.
  [https://crates.io/crates/sqlx]
- React prevents HTML injection by default.

### Insecure Design

- Only popular cryptographic libraries were used.
- Rust is memory safe.

### Security Misconfiguration

It is important to note that a default admin account is generated.
The password of this account should be changed immediately when deploying.

- The system can be deployed as docker containers.

### Identification and Authentication Failures

- A very simple weak password check is enforced on the frontend.
  If people really want they can override this but the average person does not know how to do this.
- No sesssion IDs are exposed in any URL.
- Session IDs are long and generated with a cryptographic random function.
- Account enumeration is made difficult by the use of random UUIDs.
- Only secure SSH key algorithms are supported.
- Login takes longer on each failure.

### Software and Data Integrity Failures

- Libraries downloaded from crates.io are sent securily.
- For yarn: [https://yarnpkg.com/features/security].
- Github CI is used for CI.
  A secret key is used to write to the cachix cache.
  This key is only stored in the secrets of this repository.
- Data is sent over HTTPS.

### Security Logging and Monitoring Failures

- Access failures are logged.
- Logs are made with `tracing`, the standard for async logging in rust.

### Server Side Request Forgery

- All user data is validated when parsing.
- HTTP is disabled when using caddy configured for HTTP.
