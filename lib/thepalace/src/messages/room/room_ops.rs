//! Room operation messages
//!
//! This module contains messages for room navigation and description:
//! - RoomGotoMsg: Client requests to move to a different room
//! - RoomDescMsg: Server describes a room
//! - RoomDescEndMsg: Marks end of room description sequence

use bytes::{Buf, BufMut};

use crate::messages::{MessageId, MessagePayload};

use super::records::RoomRec;

/// MessageId::RoomGoto - Client requests to move to a different room
///
/// Sent from client to server to request moving to a different room.
/// If successful, server sends:
/// - MessageId::UserExit to users in old room
/// - MessageId::UserNew to users in new room
/// - MessageId::RoomDesc to the client (describing new room)
/// - MessageId::UserList to the client (users in new room)
/// - MessageId::RoomDescEND to the client (end of description)
///
/// If unsuccessful, server sends MessageId::NavError.
///
/// Format:
/// - dest: RoomID (2 bytes, i16)
#[derive(Debug, Clone, PartialEq)]
pub struct RoomGotoMsg {
    pub dest: i16,
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

/// MessageId::RoomDescEND - Marks end of room description sequence
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

/// MessageId::RoomDesc - Server describes a room to the client
///
/// This is one of the most complex messages in the Palace protocol.
/// It contains complete room information including metadata, hotspots,
/// pictures, loose props, and draw commands.
///
/// Sent from server to client when:
/// - User enters a new room (after MessageId::RoomGoto)
/// - Room description is updated (after MessageId::RoomSetDesc)
///
/// The client should parse this message and render the room accordingly.
///
/// Format:
/// - room: RoomRec (42 bytes fixed + variable data)
#[derive(Debug, Clone, PartialEq)]
pub struct RoomDescMsg {
    pub room: RoomRec,
}

impl RoomDescMsg {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            room: RoomRec::from_bytes(buf)?,
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        self.room.to_bytes(buf);
    }
}

impl MessagePayload for RoomDescMsg {
    fn message_id() -> MessageId {
        MessageId::RoomDesc
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Self::from_bytes(buf)
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        self.to_bytes(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::flags::RoomFlags;
    use bytes::{Bytes, BytesMut};

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

    #[test]
    fn test_room_desc_msg_payload_trait() {
        // Create minimal RoomRec for testing
        let room = RoomRec {
            room_flags: RoomFlags::empty(),
            faces_id: 0,
            room_id: 1,
            room_name_ofst: -1,
            pict_name_ofst: -1,
            artist_name_ofst: -1,
            password_ofst: -1,
            nbr_hotspots: 0,
            hotspot_ofst: 0,
            nbr_pictures: 0,
            picture_ofst: 0,
            nbr_draw_cmds: 0,
            first_draw_cmd: 0,
            nbr_people: 0,
            nbr_lprops: 0,
            first_lprop: 0,
            len_vars: 0,
            var_buf: Bytes::new(),
        };

        let msg = RoomDescMsg { room };

        // Test to_message()
        let message = msg.to_message(0);
        assert_eq!(message.msg_id, MessageId::RoomDesc);
        assert_eq!(message.ref_num, 0);

        // Test parse_payload()
        let parsed = message.parse_payload::<RoomDescMsg>().unwrap();
        assert_eq!(parsed.room.room_id, msg.room.room_id);
    }
}
