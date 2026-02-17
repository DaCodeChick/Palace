//! Room message payloads
//!
//! This module implements message structures for room-related operations:
//! - MSG_ROOMGOTO: Client requests to move to a different room
//! - MSG_ROOMDESC: Server describes a room (complex structure with RoomRec)
//! - MSG_ROOMDESCEND: Marks end of room description sequence
//! - MSG_ROOMNEW: Create a new room
//! - MSG_ROOMSETDESC: Update room description
//!
//! Note: RoomRec and related structures are complex and will be implemented
//! in phases. For now, we implement the simpler room messages.

use bytes::{Buf, BufMut};

use crate::messages::{MessageId, MessagePayload};
use crate::RoomID;

/// MSG_ROOMGOTO - Client requests to move to a different room
///
/// Sent from client to server to request moving to a different room.
/// If successful, server sends:
/// - MSG_USEREXIT to users in old room
/// - MSG_USERNEW to users in new room
/// - MSG_ROOMDESC to the client (describing new room)
/// - MSG_USERLIST to the client (users in new room)
/// - MSG_ROOMDESCEND to the client (end of description)
///
/// If unsuccessful, server sends MSG_NAVERROR.
///
/// Format:
/// - dest: RoomID (2 bytes, i16)
#[derive(Debug, Clone, PartialEq)]
pub struct RoomGotoMsg {
    pub dest: RoomID,
}

impl RoomGotoMsg {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            dest: buf.get_i16(),
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i16(self.dest);
    }
}

impl MessagePayload for RoomGotoMsg {
    fn message_id() -> MessageId {
        MessageId::RoomGoto
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Self::from_bytes(buf)
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        self.to_bytes(buf);
    }
}

/// MSG_ROOMDESCEND - Marks end of room description sequence
///
/// Sent from server to client to indicate that all room description
/// messages have been sent and the client can now render the room.
///
/// This message has no payload - just the 12-byte header.
#[derive(Debug, Clone, PartialEq)]
pub struct RoomDescEndMsg;

impl RoomDescEndMsg {
    pub fn from_bytes(_buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self)
    }

    pub fn to_bytes(&self, _buf: &mut impl BufMut) {
        // Empty payload
    }
}

impl MessagePayload for RoomDescEndMsg {
    fn message_id() -> MessageId {
        MessageId::RoomDescEnd
    }

    fn from_bytes(_buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self)
    }

    fn to_bytes(&self, _buf: &mut impl BufMut) {
        // Empty payload
    }
}

// TODO: Implement RoomRec and related structures for MSG_ROOMDESC
// RoomRec is complex with variable-length data including:
// - Room metadata (flags, IDs, offsets)
// - Hotspots (clickable areas with scripts)
// - Pictures (background layers)
// - Draw commands
// - Variable-length buffer with strings and data
//
// This will require:
// - Hotspot structure
// - Picture structure
// - Draw command structure
// - Variable buffer parsing utilities

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_room_goto_msg_roundtrip() {
        let msg = RoomGotoMsg { dest: 42 };

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        assert_eq!(buf.len(), 2); // i16

        let mut reader = buf.freeze();
        let parsed = RoomGotoMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_room_goto_msg_payload_trait() {
        let msg = RoomGotoMsg { dest: 42 };

        // Test to_message()
        let message = msg.to_message(0);
        assert_eq!(message.msg_id, MessageId::RoomGoto);
        assert_eq!(message.ref_num, 0);

        // Test parse_payload()
        let parsed = message.parse_payload::<RoomGotoMsg>().unwrap();
        assert_eq!(parsed.dest, msg.dest);
    }

    #[test]
    fn test_room_desc_end_msg() {
        let msg = RoomDescEndMsg;

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        assert_eq!(buf.len(), 0); // Empty payload

        let mut reader = buf.freeze();
        let parsed = RoomDescEndMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_room_desc_end_msg_payload_trait() {
        let msg = RoomDescEndMsg;

        // Test to_message()
        let message = msg.to_message(0);
        assert_eq!(message.msg_id, MessageId::RoomDescEnd);
        assert_eq!(message.ref_num, 0);

        // Test parse_payload()
        let parsed = message.parse_payload::<RoomDescEndMsg>().unwrap();
        assert_eq!(parsed, msg);
    }
}
