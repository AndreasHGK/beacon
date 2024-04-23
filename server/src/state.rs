use std::sync::Arc;

use axum::extract::FromRef;
use sqlx::PgPool;

use super::file::FileDb;

/// Combines all different state types into one.
#[derive(Clone, FromRef)]
pub struct AppState {
    pub database: PgPool,
    pub file_store: Arc<FileDb>,
}
