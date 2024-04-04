use std::sync::Arc;

use axum::extract::FromRef;
use leptos::LeptosOptions;
use sqlx::PgPool;

use super::file::FileDb;

/// Combines all different state types into one.
#[derive(Clone, FromRef)]
pub struct AppState {
    pub leptos_options: Arc<LeptosOptions>,
    pub database: PgPool,
    pub file_store: Arc<FileDb>,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(input: &AppState) -> Self {
        input.leptos_options.as_ref().clone()
    }
}
