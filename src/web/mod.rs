pub mod handlers;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::services::ServeDir;
use std::net::SocketAddr;
use crate::web::handlers::{get_dashboard, post_update_program, AppState};
use std::sync::Arc;
use minijinja::Environment;
use sqlx::SqlitePool;
use std::fs;

pub async fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(get_dashboard))
        .route("/update/:name", post(post_update_program))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state)
}

pub async fn run_server(pool: SqlitePool, port: u16) -> anyhow::Result<()> {
    let mut jinja = Environment::new();
    let template_content = fs::read_to_string("templates/index.html")?;
    let static_template: &'static str = Box::leak(template_content.into_boxed_str());
    jinja.add_template("index.html", static_template)?;

    let state = AppState {
        db_pool: pool,
        jinja: Arc::new(jinja),
    };

    let app = create_router(state).await;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Web server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
