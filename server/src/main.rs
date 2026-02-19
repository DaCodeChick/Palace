//! Palace Server - Main entry point

mod config;
mod db;
mod net;
mod state;

use anyhow::{Context, Result};
use config::Config;
use db::Database;
use net::handler::ConnectionHandler;
use state::ServerState;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("Palace Server starting...");

    // Load configuration from file if it exists, otherwise use defaults
    let config = if std::path::Path::new("palace.json").exists() {
        info!("Loading configuration from palace.json");
        Config::from_file("palace.json")?
    } else {
        info!("Using default configuration (palace.json not found)");
        Config::default()
    };
    
    info!("Server configuration: {:?}", config);

    // Connect to database
    let db_url = format!("sqlite:{}", config.database.path);
    let db = Database::new(&db_url)
        .await
        .context("Failed to connect to database")?;

    // Initialize database schema
    db.init_schema()
        .await
        .context("Failed to initialize database schema")?;

    // Initialize server state
    let state = ServerState::new(db);
    info!("Server state initialized");

    // Bind TCP listener
    let bind_addr = config.bind_addr()?;
    let listener = TcpListener::bind(&bind_addr)
        .await
        .context("Failed to bind TCP listener")?;

    info!("Listening on {}", bind_addr);

    // Accept connections
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                info!("New connection from {}", addr);
                let state = state.clone();

                // Spawn a task for this connection
                tokio::spawn(async move {
                    let handler = ConnectionHandler::new(socket, addr, state);
                    if let Err(e) = handler.handle().await {
                        error!("Connection error from {}: {}", addr, e);
                    }
                    info!("Connection closed: {}", addr);
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}
