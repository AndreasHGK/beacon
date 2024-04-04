use std::sync::Arc;

use axum::extract::FromRef;
use leptos::{server_fn::error::NoCustomError, use_context, LeptosOptions, ServerFnError};
use leptos_router::RouteListing;
use sqlx::PgPool;

use super::file::FileDb;

/// Combines all different state types into one.
#[derive(Clone, FromRef)]
pub struct AppState {
    pub leptos_options: Arc<LeptosOptions>,
    pub routes: Arc<Vec<RouteListing>>,
    pub database: PgPool,
    pub file_store: Arc<FileDb>,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(input: &AppState) -> Self {
        input.leptos_options.as_ref().clone()
    }
}

pub fn get_state<S: FromRef<AppState>>() -> Result<S, ServerFnError> {
    let s = use_context::<AppState>()
        .ok_or_else(|| ServerFnError::<NoCustomError>::ServerError("state missing".into()))?;
    Ok(S::from_ref(&s))
}
