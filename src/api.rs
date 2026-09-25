use axum::{
    Json, Router,
    extract::State,
    response::IntoResponse,
    routing::get,
};
use std::net::SocketAddr;

use crate::app::{DashboardState, SharedState};

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/status", get(status))
        .with_state(state)
}

pub async fn run_server(state: SharedState, addr: SocketAddr) -> Result<(), std::io::Error> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router(state))
        .await
        .map_err(std::io::Error::other)
}

async fn root() -> impl IntoResponse {
    "System Monitor API"
}

async fn health() -> impl IntoResponse {
    "ok"
}

async fn status(State(state): State<SharedState>) -> Json<DashboardState> {
    Json(state.read().await.clone())
}
