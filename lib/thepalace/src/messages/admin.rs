//! Admin and server management message payloads
//!
//! This module implements message structures for administrative operations:
//! - MSG_SUPERUSER: Enter wizard/god mode with password
//! - MSG_KILLUSER: Forcibly disconnect a user
//! - MSG_SERVERDOWN: Server shutdown/disconnect notification

use bytes::{Buf, BufMut};

use crate::buffer::{BufExt, BufMutExt};
use crate::messages::{MessageId, MessagePayload};
use crate::UserID;

/// MSG_SUPERUSER - Request to enter superuser (wizard/god) mode
///
/// Client sends password to server. If correct, server responds with
/// MSG_USERSTATUS granting elevated privileges.
#[derive(Debug, Clone, PartialEq)]
pub struct SuperUserMsg {
    /// Password for wizard or god mode
    pub password: String,
}

impl SuperUserMsg {
    /// Create a new SuperUserMsg
    pub fn new(password: impl Into<String>) -> Self {
        Self {
            password: password.into(),
        }
    }
}

impl MessagePayload for SuperUserMsg {
    fn message_id() -> MessageId {
        MessageId::SuperUser
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            password: buf.get_pstring()?,
        })
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_pstring(&self.password);
    }
}

/// MSG_KILLUSER - Request to forcibly disconnect a user
///
/// Client (with sufficient authority) sends this to kick a user off the server.
#[derive(Debug, Clone, PartialEq)]
pub struct KillUserMsg {
    /// UserID of the user to disconnect
    pub target_id: UserID,
}

impl KillUserMsg {
    /// Create a new KillUserMsg
    pub fn new(target_id: UserID) -> Self {
        Self { target_id }
    }
}

impl MessagePayload for KillUserMsg {
    fn message_id() -> MessageId {
        MessageId::KillUser
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            target_id: buf.get_i32(),
        })
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i32(self.target_id);
    }
}

/// Reason codes for MSG_SERVERDOWN
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ServerDownReason {
    /// Unknown reason
    Unknown = 0,
    /// User logged off normally
    LoggedOff = 1,
    /// Communication error
    CommError = 2,
    /// User flooded server with too many messages
    Flood = 3,
    /// Killed by another player
    KilledByPlayer = 4,
    /// Server is shutting down
    ServerDown = 5,
    /// User unresponsive to pings
    Unresponsive = 6,
    /// Killed by system operator
    KilledBySysop = 7,
    /// Server is full
    ServerFull = 8,
    /// Invalid serial number
    InvalidSerialNumber = 9,
    /// Duplicate user already connected
    DuplicateUser = 10,
    /// Death penalty active
    DeathPenaltyActive = 11,
    /// User is banished
    Banished = 12,
    /// Banish kill
    BanishKill = 13,
    /// No guests allowed
    NoGuests = 14,
    /// Demo expired
    DemoExpired = 15,
    /// Verbose reason (see message field)
    Verbose = 16,
}

impl ServerDownReason {
    /// Convert from i32 refNum value
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Unknown),
            1 => Some(Self::LoggedOff),
            2 => Some(Self::CommError),
            3 => Some(Self::Flood),
            4 => Some(Self::KilledByPlayer),
            5 => Some(Self::ServerDown),
            6 => Some(Self::Unresponsive),
            7 => Some(Self::KilledBySysop),
            8 => Some(Self::ServerFull),
            9 => Some(Self::InvalidSerialNumber),
            10 => Some(Self::DuplicateUser),
            11 => Some(Self::DeathPenaltyActive),
            12 => Some(Self::Banished),
            13 => Some(Self::BanishKill),
            14 => Some(Self::NoGuests),
            15 => Some(Self::DemoExpired),
            16 => Some(Self::Verbose),
            _ => None,
        }
    }

    /// Convert to i32 refNum value
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

/// MSG_SERVERDOWN - Server disconnect notification
///
/// Server sends this to inform client that connection is being dropped.
/// The reason is encoded in the Message's refNum field.
/// If reason is Verbose, the message contains a CString explanation.
#[derive(Debug, Clone, PartialEq)]
pub struct ServerDownMsg {
    /// Optional verbose reason (only if reason is Verbose)
    pub reason_text: Option<String>,
}

impl ServerDownMsg {
    /// Create a new ServerDownMsg without verbose reason
    pub fn new() -> Self {
        Self { reason_text: None }
    }

    /// Create a ServerDownMsg with verbose reason text
    pub fn with_reason(reason_text: impl Into<String>) -> Self {
        Self {
            reason_text: Some(reason_text.into()),
        }
    }

    /// Check if this has a verbose reason
    pub fn is_verbose(&self) -> bool {
        self.reason_text.is_some()
    }
}

impl Default for ServerDownMsg {
    fn default() -> Self {
        Self::new()
    }
}

impl MessagePayload for ServerDownMsg {
    fn message_id() -> MessageId {
        MessageId::ServerDown
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        if buf.has_remaining() {
            Ok(Self {
                reason_text: Some(buf.get_cstring()?),
            })
        } else {
            Ok(Self { reason_text: None })
        }
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        if let Some(ref text) = self.reason_text {
            buf.put_cstring(text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_super_user_msg() {
        let msg = SuperUserMsg::new("secret123");

        let mut buf = vec![];
        msg.to_bytes(&mut buf);
        assert!(buf.len() > 0);

        let parsed = SuperUserMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed.password, "secret123");
    }

    #[test]
    fn test_kill_user_msg() {
        let msg = KillUserMsg::new(12345);

        let mut buf = vec![];
        msg.to_bytes(&mut buf);
        assert_eq!(buf.len(), 4);

        let parsed = KillUserMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed.target_id, 12345);
    }

    #[test]
    fn test_server_down_reason_conversions() {
        assert_eq!(ServerDownReason::LoggedOff.as_i32(), 1);
        assert_eq!(ServerDownReason::ServerDown.as_i32(), 5);
        assert_eq!(ServerDownReason::Verbose.as_i32(), 16);

        assert_eq!(
            ServerDownReason::from_i32(1),
            Some(ServerDownReason::LoggedOff)
        );
        assert_eq!(
            ServerDownReason::from_i32(5),
            Some(ServerDownReason::ServerDown)
        );
        assert_eq!(ServerDownReason::from_i32(999), None);
    }

    #[test]
    fn test_server_down_msg_simple() {
        let msg = ServerDownMsg::new();
        assert!(!msg.is_verbose());

        let mut buf = vec![];
        msg.to_bytes(&mut buf);
        assert_eq!(buf.len(), 0);

        let parsed = ServerDownMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed.reason_text, None);
    }

    #[test]
    fn test_server_down_msg_verbose() {
        let msg = ServerDownMsg::with_reason("Server maintenance in progress");
        assert!(msg.is_verbose());

        let mut buf = vec![];
        msg.to_bytes(&mut buf);
        assert!(buf.len() > 0);

        let parsed = ServerDownMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(
            parsed.reason_text.as_deref(),
            Some("Server maintenance in progress")
        );
    }
}
