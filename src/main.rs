mod api;
mod app;
mod tui;

use std::net::SocketAddr;

use api::run_server;
use app::{create_state, run_state_updater};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let state = create_state(addr.to_string());

    let updater_state = state.clone();
    tokio::spawn(async move {
        run_state_updater(updater_state).await;
    });

    let server_state = state.clone();
    tokio::spawn(async move {
        if let Err(error) = run_server(server_state, addr).await {
            eprintln!("server error: {error}");
        }
    });

    tui::run_tui(state).await?;

    Ok(())
}
