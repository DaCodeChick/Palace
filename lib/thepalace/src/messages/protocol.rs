//! Protocol and status message payloads
//!
//! This module implements message structures for protocol negotiation and status:
//! - MSG_VERSION: Server version identification
//! - MSG_USERSTATUS: User status flag updates
//! - MSG_NAVERROR: Navigation error notifications

use bytes::{Buf, BufMut};

use crate::messages::{MessageId, MessagePayload};

// ============================================================================
// Version Message
// ============================================================================

/// MSG_VERSION
///
/// Server-to-client: Sent during logon to identify the server version
///
/// The version number is encoded in the message's refNum field:
/// - High 16 bits: Major version
/// - Low 16 bits: Minor version
///
/// This message has no payload body (length = 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionMsg;

impl MessagePayload for VersionMsg {
    fn message_id() -> MessageId {
        MessageId::Version
    }

    fn from_bytes(_buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self)
    }

    fn to_bytes(&self, _buf: &mut impl BufMut) {
        // Empty payload
    }
}

// ============================================================================
// User Status Message
// ============================================================================

/// MSG_USERSTATUS
///
/// Server-to-client: Updates the client about the user's status
///
/// The UserID is in the message's refNum field.
///
/// Contains:
/// - flags: Status bit flags (see UserFlags in MSG_LISTOFALLUSERS)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserStatusMsg {
    pub flags: i16,
}

impl UserStatusMsg {
    /// Create a new UserStatusMsg
    pub fn new(flags: i16) -> Self {
        Self { flags }
    }
}

impl MessagePayload for UserStatusMsg {
    fn message_id() -> MessageId {
        MessageId::UserStatus
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            flags: buf.get_i16(),
        })
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i16(self.flags);
    }
}

// ============================================================================
// Navigation Error Message
// ============================================================================

/// Navigation error codes
///
/// These codes indicate the nature of a room navigation failure.
/// The error code is stored in the message's refNum field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum NavErrorCode {
    /// Internal error
    InternalError = 0,
    /// Room ID is unknown/doesn't exist
    RoomUnknown = 1,
    /// Room is full (at capacity)
    RoomFull = 2,
    /// Room is closed (access denied)
    RoomClosed = 3,
    /// User cannot author in this room
    CantAuthor = 4,
    /// Palace server is full
    PalaceFull = 5,
}

impl NavErrorCode {
    /// Convert from i32 to NavErrorCode
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::InternalError),
            1 => Some(Self::RoomUnknown),
            2 => Some(Self::RoomFull),
            3 => Some(Self::RoomClosed),
            4 => Some(Self::CantAuthor),
            5 => Some(Self::PalaceFull),
            _ => None,
        }
    }

    /// Convert to i32
    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

/// MSG_NAVERROR
///
/// Server-to-client: Informs the client about a navigation failure
///
/// The error code is in the message's refNum field (NavErrorCode enum).
///
/// This message has no payload body (length = 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavErrorMsg;

impl MessagePayload for NavErrorMsg {
    fn message_id() -> MessageId {
        MessageId::NavError
    }

    fn from_bytes(_buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self)
    }

    fn to_bytes(&self, _buf: &mut impl BufMut) {
        // Empty payload
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_msg() {
        let msg = VersionMsg;

        let mut buf = vec![];
        msg.to_bytes(&mut buf);
        assert_eq!(buf.len(), 0); // Empty payload

        let parsed = VersionMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_user_status_msg() {
        let msg = UserStatusMsg::new(0x0001); // Some flag value

        let mut buf = vec![];
        msg.to_bytes(&mut buf);
        assert_eq!(buf.len(), 2);

        let parsed = UserStatusMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed.flags, 0x0001);
    }

    #[test]
    fn test_nav_error_msg() {
        let msg = NavErrorMsg;

        let mut buf = vec![];
        msg.to_bytes(&mut buf);
        assert_eq!(buf.len(), 0); // Empty payload

        let parsed = NavErrorMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_nav_error_code_conversion() {
        assert_eq!(NavErrorCode::from_i32(0), Some(NavErrorCode::InternalError));
        assert_eq!(NavErrorCode::from_i32(1), Some(NavErrorCode::RoomUnknown));
        assert_eq!(NavErrorCode::from_i32(2), Some(NavErrorCode::RoomFull));
        assert_eq!(NavErrorCode::from_i32(3), Some(NavErrorCode::RoomClosed));
        assert_eq!(NavErrorCode::from_i32(4), Some(NavErrorCode::CantAuthor));
        assert_eq!(NavErrorCode::from_i32(5), Some(NavErrorCode::PalaceFull));
        assert_eq!(NavErrorCode::from_i32(99), None);

        assert_eq!(NavErrorCode::InternalError.to_i32(), 0);
        assert_eq!(NavErrorCode::RoomUnknown.to_i32(), 1);
        assert_eq!(NavErrorCode::RoomFull.to_i32(), 2);
        assert_eq!(NavErrorCode::RoomClosed.to_i32(), 3);
        assert_eq!(NavErrorCode::CantAuthor.to_i32(), 4);
        assert_eq!(NavErrorCode::PalaceFull.to_i32(), 5);
    }
}
