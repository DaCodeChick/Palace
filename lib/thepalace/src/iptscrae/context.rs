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

    /// Set the user's color (SETCOLOR).
    fn set_color(&mut self, color: i16);

    /// Set user props (SETPROPS).
    fn set_props(&mut self, props: Vec<AssetSpec>);

    /// Set the user's position (SETPOS).
    fn set_pos(&mut self, x: i16, y: i16);

    /// Move the user relative to current position (MOVE).
    fn move_user(&mut self, dx: i16, dy: i16);

    /// Navigate to a URL (GOTOURL).
    fn goto_url(&mut self, url: &str);

    /// Navigate to a URL in a specific frame (GOTOURLFRAME).
    fn goto_url_frame(&mut self, url: &str, frame: &str);

    /// Send a global message to all users on the server (GLOBALMSG).
    fn global_msg(&mut self, message: &str);

    /// Send a status message (STATUSMSG).
    fn status_msg(&mut self, message: &str);

    /// Send a superuser message (SUSRMSG).
    fn superuser_msg(&mut self, message: &str);

    /// Log a message (LOGMSG).
    fn log_msg(&mut self, message: &str);

    /// Set spot state (SETSPOTSTATE).
    fn set_spot_state(&mut self, spot_id: i32, state: i32);

    /// Add a loose prop to the room (ADDLOOSEPROP).
    fn add_loose_prop(&mut self, prop_id: i32, x: i16, y: i16);

    /// Clear all loose props from the room (CLEARLOOSEPROPS).
    fn clear_loose_props(&mut self);
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
    fn set_color(&mut self, _color: i16) {}
    fn set_props(&mut self, _props: Vec<AssetSpec>) {}
    fn set_pos(&mut self, _x: i16, _y: i16) {}
    fn move_user(&mut self, _dx: i16, _dy: i16) {}
    fn goto_url(&mut self, _url: &str) {}
    fn goto_url_frame(&mut self, _url: &str, _frame: &str) {}
    fn global_msg(&mut self, _message: &str) {}
    fn status_msg(&mut self, _message: &str) {}
    fn superuser_msg(&mut self, _message: &str) {}
    fn log_msg(&mut self, _message: &str) {}
    fn set_spot_state(&mut self, _spot_id: i32, _state: i32) {}
    fn add_loose_prop(&mut self, _prop_id: i32, _x: i16, _y: i16) {}
    fn clear_loose_props(&mut self) {}
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

    /// Current user color (roundhead color, 0-15).
    pub user_color: i16,

    /// Current user props.
    pub user_props: Vec<AssetSpec>,

    /// Current user position X coordinate.
    pub user_pos_x: i16,

    /// Current user position Y coordinate.
    pub user_pos_y: i16,

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
            user_color: 0,
            user_props: Vec::new(),
            user_pos_x: 0,
            user_pos_y: 0,
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
