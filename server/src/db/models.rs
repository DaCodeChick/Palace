//! Database models

use serde::{Deserialize, Serialize};

/// User record from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub user_id: i64,
    pub username: String,
    pub password_hash: Option<String>,
    pub wizard_password: Option<String>,
    pub flags: i64,
    pub registration_date: i64,
    pub last_login: Option<i64>,
}

/// Room record from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Room {
    pub room_id: i64,
    pub name: String,
    pub artist: Option<String>,
    pub background_image: Option<String>,
    pub flags: i64,
    pub max_occupancy: i64,
    pub faces_id: i64,
    pub room_data: Option<Vec<u8>>,
}

/// Prop record from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Prop {
    pub prop_id: i64,
    pub crc32: i64,
    pub name: String,
    pub flags: i64,
    pub width: i64,
    pub height: i64,
    pub file_path: String,
    pub created_at: i64,
}

/// Hotspot record from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Hotspot {
    pub hotspot_id: i64,
    pub room_id: i64,
    pub id: i64,
    pub name: Option<String>,
    pub r#type: i64,
    pub dest_room_id: Option<i64>,
    pub dest_hotspot_id: Option<i64>,
    pub loc_h: i64,
    pub loc_v: i64,
    pub script_event_mask: i64,
    pub script_text: Option<String>,
    pub state: i64,
}

/// Hotspot point (polygon vertex)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HotspotPoint {
    pub id: i64,
    pub hotspot_id: i64,
    pub point_order: i64,
    pub pos_h: i64,
    pub pos_v: i64,
}

/// Ban record from database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Ban {
    pub ban_id: i64,
    pub user_id: Option<i64>,
    pub ip_address: Option<String>,
    pub reason: Option<String>,
    pub banned_at: i64,
    pub expires_at: Option<i64>,
    pub banned_by_user_id: Option<i64>,
}
