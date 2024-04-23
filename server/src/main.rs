use std::{env, net::SocketAddr, sync::Arc};

use anyhow::Context;
use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

use crate::{
    api::{download::file_content, file_info::file_info, upload::handle_upload},
    file::{FileDb, FileStore},
    state::AppState,
};

mod api;
mod file;
mod state;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env()
                .context("failed to configure logger")?,
        )
        .init();

    let bind_addr: SocketAddr = env::var("BEACON_SERVER_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:4000".to_string())
        .parse()
        .context("failed to parse bind address")?;

    info!("Connecting to database.");
    let pool = PgPool::connect(
        &env::var("DATABASE_URL").context("could not read `DATABASE_URL` environment variable")?,
    )
    .await
    .context("could not connect to database")?;

    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .context("error while migrating database")?;

    info!("Opening file store.");
    let file_store = FileStore::new(
        env::var("FILE_STORE_ROOT")
            .context("could not read `FILE_STORE_ROOT` environment variable")?,
    )
    .await
    .context("could not create file store")?;

    let file_db = FileDb::new(pool.clone(), file_store);

    let state = AppState {
        database: pool,
        file_store: Arc::new(file_db),
    };

    let app = Router::new()
        .route("/files", post(handle_upload))
        .route("/files/:file_id/:file_name", get(file_info))
        .route("/files/:file_id/:file_name/content", get(file_content))
        .with_state(state);

    info!("Listening on `http://{}`.", &bind_addr);
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
