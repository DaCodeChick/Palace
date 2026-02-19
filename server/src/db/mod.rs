//! Database layer for Palace server

pub mod models;
pub mod users;
pub mod rooms;

use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use tracing::info;

/// Database connection pool
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection with the given path
    pub async fn new(database_path: &str) -> Result<Self> {
        info!("Connecting to database: {}", database_path);

        // Create options with proper settings
        let options = SqliteConnectOptions::from_str(database_path)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        // Create connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect_with(options)
            .await
            .context("Failed to connect to database")?;

        info!("Database connection established");

        Ok(Self { pool })
    }

    /// Initialize database schema
    pub async fn init_schema(&self) -> Result<()> {
        info!("Initializing database schema");

        // Check if schema already exists
        let table_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='users'"
        )
        .fetch_one(&self.pool)
        .await?;

        if table_count > 0 {
            info!("Database schema already exists, skipping initialization");
            return Ok(());
        }

        // Create all tables
        sqlx::query(
            r#"
            -- Users table
            CREATE TABLE users (
                user_id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE COLLATE NOCASE,
                password_hash TEXT,
                wizard_password TEXT,
                flags INTEGER NOT NULL DEFAULT 8,
                registration_date INTEGER NOT NULL,
                last_login INTEGER
            );

            -- Create index on username for faster lookups
            CREATE INDEX idx_users_username ON users(username);
            "#
        )
        .execute(&self.pool)
        .await
        .context("Failed to create users table")?;

        sqlx::query(
            r#"
            -- Rooms table
            CREATE TABLE rooms (
                room_id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                artist TEXT,
                background_image TEXT,
                flags INTEGER NOT NULL DEFAULT 0,
                max_occupancy INTEGER DEFAULT 0,
                faces_id INTEGER DEFAULT 0,
                room_data BLOB
            );
            "#
        )
        .execute(&self.pool)
        .await
        .context("Failed to create rooms table")?;

        sqlx::query(
            r#"
            -- Props registry
            CREATE TABLE props (
                prop_id INTEGER PRIMARY KEY AUTOINCREMENT,
                crc32 INTEGER NOT NULL UNIQUE,
                name TEXT NOT NULL,
                flags INTEGER NOT NULL,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );

            -- Create index on CRC32 for asset lookups
            CREATE INDEX idx_props_crc32 ON props(crc32);
            "#
        )
        .execute(&self.pool)
        .await
        .context("Failed to create props table")?;

        sqlx::query(
            r#"
            -- Loose props in rooms
            CREATE TABLE room_loose_props (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                room_id INTEGER NOT NULL,
                prop_id INTEGER NOT NULL,
                pos_h INTEGER NOT NULL,
                pos_v INTEGER NOT NULL,
                FOREIGN KEY (room_id) REFERENCES rooms(room_id) ON DELETE CASCADE,
                FOREIGN KEY (prop_id) REFERENCES props(prop_id) ON DELETE CASCADE
            );

            -- Create index for faster room prop queries
            CREATE INDEX idx_room_loose_props_room ON room_loose_props(room_id);
            "#
        )
        .execute(&self.pool)
        .await
        .context("Failed to create room_loose_props table")?;

        sqlx::query(
            r#"
            -- Hotspots table
            CREATE TABLE hotspots (
                hotspot_id INTEGER PRIMARY KEY AUTOINCREMENT,
                room_id INTEGER NOT NULL,
                id INTEGER NOT NULL,
                name TEXT,
                type INTEGER NOT NULL,
                dest_room_id INTEGER,
                dest_hotspot_id INTEGER,
                loc_h INTEGER NOT NULL,
                loc_v INTEGER NOT NULL,
                script_event_mask INTEGER NOT NULL DEFAULT 0,
                script_text TEXT,
                state INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (room_id) REFERENCES rooms(room_id) ON DELETE CASCADE
            );

            -- Create index for room hotspot queries
            CREATE INDEX idx_hotspots_room ON hotspots(room_id);
            "#
        )
        .execute(&self.pool)
        .await
        .context("Failed to create hotspots table")?;

        sqlx::query(
            r#"
            -- Hotspot points (polygon vertices)
            CREATE TABLE hotspot_points (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                hotspot_id INTEGER NOT NULL,
                point_order INTEGER NOT NULL,
                pos_h INTEGER NOT NULL,
                pos_v INTEGER NOT NULL,
                FOREIGN KEY (hotspot_id) REFERENCES hotspots(hotspot_id) ON DELETE CASCADE
            );

            CREATE INDEX idx_hotspot_points_hotspot ON hotspot_points(hotspot_id);
            "#
        )
        .execute(&self.pool)
        .await
        .context("Failed to create hotspot_points table")?;

        sqlx::query(
            r#"
            -- Ban list
            CREATE TABLE bans (
                ban_id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER,
                ip_address TEXT,
                reason TEXT,
                banned_at INTEGER NOT NULL,
                expires_at INTEGER,
                banned_by_user_id INTEGER,
                FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
            );

            -- Create index for active ban checks
            CREATE INDEX idx_bans_user ON bans(user_id);
            CREATE INDEX idx_bans_ip ON bans(ip_address);
            "#
        )
        .execute(&self.pool)
        .await
        .context("Failed to create bans table")?;

        // Insert default rooms
        sqlx::query(
            r#"
            INSERT INTO rooms (room_id, name, artist, flags, max_occupancy) VALUES
                (0, 'Gate', 'System', 0, 50),
                (1, 'Main Hall', 'System', 0, 100),
                (2, 'Ballroom', 'System', 0, 75);
            "#
        )
        .execute(&self.pool)
        .await
        .context("Failed to insert default rooms")?;

        info!("Database schema initialized successfully");
        Ok(())
    }

    /// Get the underlying pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Close the database connection
    pub async fn close(self) {
        self.pool.close().await;
    }
}
