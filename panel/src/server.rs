use std::{env, sync::Arc};

use anyhow::Context;
use axum::{
    body::Body,
    extract::{Request, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use sqlx::PgPool;

use crate::{
    app::App,
    server::{
        api::{download::file_content, upload::handle_upload},
        file::{FileDb, FileStore},
        fileserv::file_and_error_handler,
        state::AppState,
    },
};

mod api;
pub mod file;
mod fileserv;
pub mod state;

async fn server_fn_handler(
    State(app_state): State<AppState>,
    request: Request<Body>,
) -> impl IntoResponse {
    leptos_axum::handle_server_fns_with_context(
        move || {
            provide_context(app_state.clone());
        },
        request,
    )
    .await
}

async fn leptos_routes_handler(State(app_state): State<AppState>, req: Request<Body>) -> Response {
    let handler = leptos_axum::render_route_with_context(
        app_state.leptos_options.as_ref().clone(),
        app_state.routes.as_ref().clone(),
        move || {
            provide_context(app_state.clone());
        },
        App,
    );
    handler(req).await.into_response()
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    simple_logger::init_with_level(log::Level::Debug).expect("couldn't initialize logging");

    log::info!("Connecting to database.");
    let pool = PgPool::connect(
        &env::var("DATABASE_URL").context("could not read `DATABASE_URL` environment variable")?,
    )
    .await
    .context("could not connect to database")?;

    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .context("error while migrating database")?;

    log::info!("Opening file store.");
    let file_store = FileStore::new(
        env::var("FILE_STORE_ROOT")
            .context("could not read `FILE_STORE_ROOT` environment variable")?,
    )
    .await
    .context("could not create file store")?;

    let file_db = FileDb::new(pool.clone(), file_store);

    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let state = AppState {
        leptos_options: Arc::new(leptos_options),
        routes: Arc::new(routes.clone()),
        database: pool,
        file_store: Arc::new(file_db),
    };

    let app = Router::new()
        // .leptos_routes(&state, routes, App)
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
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
