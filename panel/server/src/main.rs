use std::{env, sync::Arc};

use anyhow::Context;
use axum::{
    routing::{get, post},
    Router,
};
use beacon_panel_shared::*;
use fileserv::file_and_error_handler;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use sqlx::PgPool;

use crate::{
    download::file_content, file_store::FileStore, state::AppState, upload::handle_upload,
};

mod download;
pub mod file_store;
mod fileserv;
mod state;
mod upload;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    simple_logger::init_with_level(log::Level::Debug).expect("couldn't initialize logging");

    log::info!("Connecting to database.");
    let pool = PgPool::connect(
        &env::var("DATABASE_URL").context("could not read `DATABASE_URL` environment variable")?,
    )
    .await
    .context("could not connect to database")?;

    log::info!("Opening file store.");
    let file_store = FileStore::new(
        env::var("FILE_STORE_ROOT")
            .context("could not read `FILE_STORE_ROOT` environment variable")?,
    )
    .await
    .context("could not create file store")?;

    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let state = AppState {
        leptos_options: Arc::new(leptos_options),
        database: pool,
        file_store: Arc::new(file_store),
    };

    let app = Router::new()
        .leptos_routes(&state, routes, App)
        .fallback(file_and_error_handler)
        .route("/upload", post(handle_upload))
        .route("/files/:file_id/:file_name/content", get(file_content))
        .with_state(state);

    log::info!("Listening on `http://{}`.", &addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
