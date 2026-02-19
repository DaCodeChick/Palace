//! User database operations

use super::Database;
use crate::db::models::User;
use anyhow::{Context, Result};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

impl Database {
    /// Get a user by username
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ? COLLATE NOCASE")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to query user")?;
        Ok(user)
    }

    /// Get a user by user_id
    pub async fn get_user_by_id(&self, user_id: i64) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to query user")?;
        Ok(user)
    }

    /// Create a new user (guest or registered)
    pub async fn create_user(&self, username: &str, password_hash: Option<&str>) -> Result<i64> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let result = sqlx::query(
            "INSERT INTO users (username, password_hash, flags, registration_date, last_login) 
             VALUES (?, ?, 8, ?, ?)",
        )
        .bind(username)
        .bind(password_hash)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("Failed to create user")?;

        let user_id = result.last_insert_rowid();
        debug!("Created user '{}' with ID {}", username, user_id);
        Ok(user_id)
    }

    /// Update user's last login timestamp
    pub async fn update_last_login(&self, user_id: i64) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        sqlx::query("UPDATE users SET last_login = ? WHERE user_id = ?")
            .bind(now)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .context("Failed to update last login")?;

        Ok(())
    }

    /// Check if user is banned by IP
    pub async fn is_ip_banned(&self, ip_address: &str) -> Result<bool> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM bans 
             WHERE ip_address = ? 
             AND (expires_at IS NULL OR expires_at > ?)",
        )
        .bind(ip_address)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .context("Failed to check IP ban")?;

        Ok(count > 0)
    }

    /// Check if user is banned by user_id
    pub async fn is_user_banned(&self, user_id: i64) -> Result<bool> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM bans 
             WHERE user_id = ? 
             AND (expires_at IS NULL OR expires_at > ?)",
        )
        .bind(user_id)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .context("Failed to check user ban")?;

        Ok(count > 0)
    }
}
