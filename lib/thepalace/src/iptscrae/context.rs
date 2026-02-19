//! Execution context for Iptscrae scripts.
//!
//! This module provides the context needed for scripts to interact with the Palace server,
//! including information about the current user, room, and event, as well as callbacks
//! for performing Palace operations like navigation and chat.

use crate::iptscrae::events::EventType;
use crate::iptscrae::value::Value;
use crate::AssetSpec;
use std::collections::HashMap;

/// Security level for script execution.
///
/// Different security levels restrict which built-in functions scripts can call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityLevel {
    /// Full server privileges (room scripts).
    Server,
    /// Sandboxed cyborg scripts with restricted operations.
    Cyborg,
    /// Administrative scripts with elevated privileges.
    Admin,
}

/// Actions that scripts can perform.
///
/// This trait defines callbacks that the VM can invoke to interact with the Palace server.
pub trait ScriptActions {
    /// Send a message to the room (SAY).
    fn say(&mut self, message: &str);

    /// Send a chat message (CHAT - same as SAY).
    fn chat(&mut self, message: &str);

    /// Send a message visible only to the local user (LOCALMSG).
    fn local_msg(&mut self, message: &str);

    /// Send a message to everyone in the room (ROOMMSG).
    fn room_msg(&mut self, message: &str);

    /// Send a private message to a specific user (PRIVATEMSG).
    fn private_msg(&mut self, user_id: i32, message: &str);

    /// Navigate to a different room (GOTOROOM).
    fn goto_room(&mut self, room_id: i16);

    /// Lock a door (LOCK).
    fn lock_door(&mut self, door_id: i32);

    /// Unlock a door (UNLOCK).
    fn unlock_door(&mut self, door_id: i32);

    /// Set the user's face (SETFACE).
    fn set_face(&mut self, face_id: i16);

    /// Set user props (SETPROPS).
    fn set_props(&mut self, props: Vec<AssetSpec>);
}

/// Default implementation that does nothing (for testing).
impl ScriptActions for () {
    fn say(&mut self, _message: &str) {}
    fn chat(&mut self, _message: &str) {}
    fn local_msg(&mut self, _message: &str) {}
    fn room_msg(&mut self, _message: &str) {}
    fn private_msg(&mut self, _user_id: i32, _message: &str) {}
    fn goto_room(&mut self, _room_id: i16) {}
    fn lock_door(&mut self, _door_id: i32) {}
    fn unlock_door(&mut self, _door_id: i32) {}
    fn set_face(&mut self, _face_id: i16) {}
    fn set_props(&mut self, _props: Vec<AssetSpec>) {}
}

/// Execution context for Iptscrae scripts.
///
/// Provides information about the current user, room, and event, as well as
/// callbacks for performing Palace operations.
pub struct ScriptContext<'a> {
    /// Security level for this script execution.
    pub security_level: SecurityLevel,

    /// Current user ID.
    pub user_id: i32,

    /// Current user name.
    pub user_name: String,

    /// Current user face (avatar) ID.
    pub user_face: i16,

    /// Current user props.
    pub user_props: Vec<AssetSpec>,

    /// Current room ID.
    pub room_id: i16,

    /// Current room name.
    pub room_name: String,

    /// Event type that triggered this script.
    pub event_type: EventType,

    /// Optional event data (e.g., hotspot ID, user ID for INCHAT/OUTCHAT).
    pub event_data: HashMap<String, Value>,

    /// Callbacks for performing Palace operations.
    pub actions: &'a mut dyn ScriptActions,
}

impl<'a> ScriptContext<'a> {
    /// Create a new script context with default values.
    pub fn new(security_level: SecurityLevel, actions: &'a mut dyn ScriptActions) -> Self {
        Self {
            security_level,
            user_id: 0,
            user_name: String::new(),
            user_face: 0,
            user_props: Vec::new(),
            room_id: 0,
            room_name: String::new(),
            event_type: EventType::Select,
            event_data: HashMap::new(),
            actions,
        }
    }

    /// Check if a function is allowed at the current security level.
    pub fn is_function_allowed(&self, function_name: &str) -> bool {
        match self.security_level {
            SecurityLevel::Server | SecurityLevel::Admin => true,
            SecurityLevel::Cyborg => {
                // Cyborgs can't lock/unlock doors or force navigation
                !matches!(function_name, "LOCK" | "UNLOCK" | "GOTOROOM")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_level() {
        let mut actions = ();
        let server_ctx = ScriptContext::new(SecurityLevel::Server, &mut actions);
        assert!(server_ctx.is_function_allowed("LOCK"));
        assert!(server_ctx.is_function_allowed("GOTOROOM"));

        let mut actions2 = ();
        let cyborg_ctx = ScriptContext::new(SecurityLevel::Cyborg, &mut actions2);
        assert!(!cyborg_ctx.is_function_allowed("LOCK"));
        assert!(!cyborg_ctx.is_function_allowed("GOTOROOM"));
        assert!(cyborg_ctx.is_function_allowed("SAY"));
        assert!(cyborg_ctx.is_function_allowed("WHONAME"));
    }

    #[test]
    fn test_context_creation() {
        let mut actions = ();
        let ctx = ScriptContext::new(SecurityLevel::Server, &mut actions);
        assert_eq!(ctx.security_level, SecurityLevel::Server);
        assert_eq!(ctx.user_id, 0);
        assert_eq!(ctx.user_name, "");
        assert_eq!(ctx.room_id, 0);
    }

    #[test]
    fn test_event_data() {
        let mut actions = ();
        let mut ctx = ScriptContext::new(SecurityLevel::Server, &mut actions);
        ctx.event_data
            .insert("hotspot_id".to_string(), Value::Integer(42));
        assert_eq!(ctx.event_data.get("hotspot_id"), Some(&Value::Integer(42)));
    }
}
