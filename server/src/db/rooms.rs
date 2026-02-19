//! Room database operations

use super::Database;
use crate::db::models::{Hotspot, HotspotPoint, Room};
use anyhow::{Context, Result};

impl Database {
    /// Get a room by room_id
    pub async fn get_room(&self, room_id: i16) -> Result<Option<Room>> {
        let room = sqlx::query_as::<_, Room>("SELECT * FROM rooms WHERE room_id = ?")
            .bind(room_id as i64)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to query room")?;
        Ok(room)
    }

    /// Get all rooms
    pub async fn get_all_rooms(&self) -> Result<Vec<Room>> {
        let rooms = sqlx::query_as::<_, Room>("SELECT * FROM rooms ORDER BY room_id")
            .fetch_all(&self.pool)
            .await
            .context("Failed to query rooms")?;
        Ok(rooms)
    }

    /// Get hotspots for a room
    pub async fn get_room_hotspots(&self, room_id: i16) -> Result<Vec<Hotspot>> {
        let hotspots = sqlx::query_as::<_, Hotspot>(
            "SELECT * FROM hotspots WHERE room_id = ? ORDER BY id",
        )
        .bind(room_id as i64)
        .fetch_all(&self.pool)
        .await
        .context("Failed to query hotspots")?;
        Ok(hotspots)
    }

    /// Get points for a hotspot
    pub async fn get_hotspot_points(&self, hotspot_id: i64) -> Result<Vec<HotspotPoint>> {
        let points = sqlx::query_as::<_, HotspotPoint>(
            "SELECT * FROM hotspot_points WHERE hotspot_id = ? ORDER BY point_order",
        )
        .bind(hotspot_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to query hotspot points")?;
        Ok(points)
    }

    /// Count users currently in a room (from in-memory state, not DB)
    /// Note: This should be called from the state manager, not the database
    /// Keeping this as a placeholder for future implementation
    pub async fn get_room_user_count(&self, _room_id: i16) -> Result<i16> {
        // This will need to query the in-memory state manager
        // For now, return 0
        Ok(0)
    }
}
