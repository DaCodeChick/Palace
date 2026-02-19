//! Server configuration

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
}

/// Server network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub server_name: String,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
    pub pool_size: u32,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub allow_guests: bool,
    pub allow_cyborgs: bool,
    pub max_prop_size: u64,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

impl Config {
    /// Load configuration from a JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path).context("Failed to read config file")?;
        let config: Config =
            serde_json::from_str(&contents).context("Failed to parse config file")?;
        Ok(config)
    }

    /// Create default configuration
    pub fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 9998,
                max_connections: 100,
                server_name: "Palace Server".to_string(),
            },
            database: DatabaseConfig {
                path: "palace.db".to_string(),
                pool_size: 10,
            },
            security: SecurityConfig {
                allow_guests: true,
                allow_cyborgs: true,
                max_prop_size: 1048576, // 1MB
            },
            logging: LoggingConfig {
                level: "info".to_string(),
            },
        }
    }

    /// Get bind address for server
    pub fn bind_addr(&self) -> Result<SocketAddr> {
        let addr = format!("{}:{}", self.server.host, self.server.port);
        addr.parse()
            .context("Invalid server host/port configuration")
    }
}
