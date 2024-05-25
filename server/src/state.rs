use std::sync::Arc;

use axum::extract::FromRef;
use sqlx::PgPool;

use crate::{auth::ssh::SSHAuthState, config::Config};

use super::file::FileDb;

/// Combines all different state types into one.
#[derive(Clone, FromRef)]
pub struct AppState {
    pub database: PgPool,
    pub file_store: Arc<FileDb>,
    pub ssh_auth: Arc<SSHAuthState>,
    pub config: Arc<Config>,
}
