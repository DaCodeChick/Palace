//! Server state management
//!
//! Manages in-memory state for connected users and active sessions
//! while using database for persistent data.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info};

use crate::db::Database;

/// User ID type
pub type UserId = i64;

/// Room ID type
pub type RoomId = i16;

/// Server broadcast message
#[derive(Debug, Clone)]
pub enum ServerMessage {
    /// User joined a room
    UserJoined {
        user_id: UserId,
        room_id: RoomId,
        username: String,
    },
    /// User left a room
    UserLeft {
        user_id: UserId,
        room_id: RoomId,
    },
    /// Chat message in a room
    Chat {
        from_user_id: UserId,
        room_id: RoomId,
        message: String,
        encrypted: bool,
    },
    /// User disconnected
    UserDisconnected { user_id: UserId },
}

/// Connected user session
#[derive(Debug)]
pub struct UserSession {
    pub user_id: UserId,
    pub username: String,
    pub room_id: RoomId,
    pub addr: SocketAddr,
    /// Channel to send messages to this user's connection
    pub tx: mpsc::UnboundedSender<ServerMessage>,
}

/// Active room state (in-memory)
#[derive(Debug, Clone)]
pub struct ActiveRoom {
    pub room_id: RoomId,
    pub user_ids: Vec<UserId>,
}

/// Shared server state
#[derive(Clone)]
pub struct ServerState {
    db: Database,
    inner: Arc<RwLock<ServerStateInner>>,
}

struct ServerStateInner {
    /// Active user sessions
    sessions: HashMap<UserId, UserSession>,
    /// Active rooms with their current users
    active_rooms: HashMap<RoomId, ActiveRoom>,
}

impl ServerState {
    /// Create new server state
    pub fn new(db: Database) -> Self {
        Self {
            db,
            inner: Arc::new(RwLock::new(ServerStateInner {
                sessions: HashMap::new(),
                active_rooms: HashMap::new(),
            })),
        }
    }

    /// Get database handle
    pub fn db(&self) -> &Database {
        &self.db
    }

    /// Register a new user session
    pub async fn register_session(
        &self,
        user_id: UserId,
        username: String,
        room_id: RoomId,
        addr: SocketAddr,
        tx: mpsc::UnboundedSender<ServerMessage>,
    ) {
        let mut inner = self.inner.write().await;
        
        let session = UserSession {
            user_id,
            username: username.clone(),
            room_id,
            addr,
            tx,
        };

        inner.sessions.insert(user_id, session);
        
        // Add user to room
        let active_room = inner
            .active_rooms
            .entry(room_id)
            .or_insert_with(|| ActiveRoom {
                room_id,
                user_ids: Vec::new(),
            });
        
        if !active_room.user_ids.contains(&user_id) {
            active_room.user_ids.push(user_id);
        }

        info!(
            "Registered session: user_id={}, username='{}', room={}",
            user_id, username, room_id
        );
    }

    /// Unregister a user session
    pub async fn unregister_session(&self, user_id: UserId) {
        let mut inner = self.inner.write().await;
        
        if let Some(session) = inner.sessions.remove(&user_id) {
            // Remove from room
            if let Some(room) = inner.active_rooms.get_mut(&session.room_id) {
                room.user_ids.retain(|&id| id != user_id);
                
                // Clean up empty rooms
                if room.user_ids.is_empty() {
                    inner.active_rooms.remove(&session.room_id);
                }
            }
            
            info!("Unregistered session: user_id={}", user_id);
        }
    }

    /// Move a user to a different room
    pub async fn move_user_to_room(&self, user_id: UserId, new_room_id: RoomId) -> bool {
        let mut inner = self.inner.write().await;
        
        if let Some(session) = inner.sessions.get(&user_id) {
            let old_room_id = session.room_id;
            
            // Remove from old room
            if let Some(old_room) = inner.active_rooms.get_mut(&old_room_id) {
                old_room.user_ids.retain(|&id| id != user_id);
                if old_room.user_ids.is_empty() {
                    inner.active_rooms.remove(&old_room_id);
                }
            }
            
            // Add to new room
            let new_room = inner
                .active_rooms
                .entry(new_room_id)
                .or_insert_with(|| ActiveRoom {
                    room_id: new_room_id,
                    user_ids: Vec::new(),
                });
            
            if !new_room.user_ids.contains(&user_id) {
                new_room.user_ids.push(user_id);
            }
        }
        
        // Update session's current room
        if let Some(session) = inner.sessions.get_mut(&user_id) {
            session.room_id = new_room_id;
            debug!("Moved user {} to room {}", user_id, new_room_id);
            return true;
        }
        
        false
    }

    /// Get list of users in a room
    pub async fn get_room_users(&self, room_id: RoomId) -> Vec<(UserId, String)> {
        let inner = self.inner.read().await;
        
        if let Some(room) = inner.active_rooms.get(&room_id) {
            room.user_ids
                .iter()
                .filter_map(|&user_id| {
                    inner.sessions.get(&user_id).map(|s| (user_id, s.username.clone()))
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Broadcast a message to all users in a room
    pub async fn broadcast_to_room(&self, room_id: RoomId, message: ServerMessage) {
        let inner = self.inner.read().await;
        
        if let Some(room) = inner.active_rooms.get(&room_id) {
            let mut sent_count = 0;
            for &user_id in &room.user_ids {
                if let Some(session) = inner.sessions.get(&user_id) {
                    // Ignore send errors (user might be disconnecting)
                    if session.tx.send(message.clone()).is_ok() {
                        sent_count += 1;
                    }
                }
            }
            debug!("Broadcast to room {}: {} recipients", room_id, sent_count);
        }
    }

    /// Send a message to a specific user
    pub async fn send_to_user(&self, user_id: UserId, message: ServerMessage) {
        let inner = self.inner.read().await;
        
        if let Some(session) = inner.sessions.get(&user_id) {
            let _ = session.tx.send(message);
        }
    }

    /// Get number of users in a room
    pub async fn get_room_user_count(&self, room_id: RoomId) -> i16 {
        let inner = self.inner.read().await;
        
        inner
            .active_rooms
            .get(&room_id)
            .map(|room| room.user_ids.len() as i16)
            .unwrap_or(0)
    }

    /// Get total number of connected users
    pub async fn get_total_users(&self) -> usize {
        let inner = self.inner.read().await;
        inner.sessions.len()
    }

    /// Check if a room exists in the database
    pub async fn room_exists(&self, room_id: RoomId) -> bool {
        self.db.get_room(room_id).await.ok().flatten().is_some()
    }
}
