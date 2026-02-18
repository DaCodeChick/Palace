//! Prop operation messages
//!
//! This module contains messages for room list and prop operations:
//! - RoomListRec: Room list record
//! - ListOfAllRoomsMsg: Request/response for list of all rooms
//! - PropDelMsg: Delete a prop from the room
//! - PropMoveMsg: Move a prop to a new position
//! - PropNewMsg: Add a new prop to the room

use bytes::{Buf, BufMut};

use crate::buffer::{BufExt, BufMutExt};
use crate::messages::flags::RoomFlags;
use crate::messages::{MessageId, MessagePayload};
use crate::{AssetSpec, Point};

/// Room list record - describes a room in the room list
///
/// Variable size due to PString name field
#[derive(Debug, Clone, PartialEq)]
pub struct RoomListRec {
    /// Room ID (stored as i32 in protocol, but actually i16)
    pub room_id: i32,
    /// Room flags
    pub flags: RoomFlags,
    /// Number of users currently in room
    pub nbr_users: i16,
    /// Room name (PString with padding to align length)
    pub name: String,
}

impl RoomListRec {
    /// Parse a RoomListRec from bytes
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        let room_id = buf.get_i32();
        let flags = RoomFlags::from_bits_truncate(buf.get_i16() as u16);
        let nbr_users = buf.get_i16();
        let name = buf.get_pstring()?;

        Ok(Self {
            room_id,
            flags,
            nbr_users,
            name,
        })
    }

    /// Serialize this RoomListRec to bytes
    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i32(self.room_id);
        buf.put_i16(self.flags.bits() as i16);
        buf.put_i16(self.nbr_users);
        buf.put_pstring(&self.name);
    }
}

/// MessageId::ListOfAllRooms - Request/response for list of all rooms
///
/// In request form (client→server): empty payload
/// In response form (server→client): array of RoomListRec
/// refNum contains the number of rooms in the response
#[derive(Debug, Clone, PartialEq)]
pub struct ListOfAllRoomsMsg {
    /// Array of rooms (empty for request, populated for response)
    pub rooms: Vec<RoomListRec>,
}

impl ListOfAllRoomsMsg {
    /// Create a new request (empty)
    pub fn request() -> Self {
        Self { rooms: vec![] }
    }

    /// Create a new response with room list
    pub fn response(rooms: Vec<RoomListRec>) -> Self {
        Self { rooms }
    }

    /// Check if this is a request (empty) or response (has rooms)
    pub fn is_request(&self) -> bool {
        self.rooms.is_empty()
    }

    /// Number of rooms
    pub fn count(&self) -> usize {
        self.rooms.len()
    }
}

impl MessagePayload for ListOfAllRoomsMsg {
    fn message_id() -> MessageId {
        MessageId::ListOfAllRooms
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        let mut rooms = Vec::new();
        while buf.has_remaining() {
            rooms.push(RoomListRec::from_bytes(buf)?);
        }
        Ok(Self { rooms })
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        for room in &self.rooms {
            room.to_bytes(buf);
        }
    }
}

/// MessageId::PropDel - Delete a prop from the room
///
/// propNum identifies the prop to delete (0-indexed in order added)
/// propNum = -1 means delete all props in the room
#[derive(Debug, Clone, PartialEq)]
pub struct PropDelMsg {
    /// Prop number to delete (-1 = all props)
    pub prop_num: i32,
}

impl PropDelMsg {
    /// Create a new PropDelMsg
    pub fn new(prop_num: i32) -> Self {
        Self { prop_num }
    }

    /// Create message to delete all props
    pub fn delete_all() -> Self {
        Self { prop_num: -1 }
    }
}

impl MessagePayload for PropDelMsg {
    fn message_id() -> MessageId {
        MessageId::PropDel
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            prop_num: buf.get_i32(),
        })
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i32(self.prop_num);
    }
}

/// MessageId::PropMove - Move a prop to a new position
///
/// propNum identifies the prop to move (0-indexed in order added)
#[derive(Debug, Clone, PartialEq)]
pub struct PropMoveMsg {
    /// Prop number to move
    pub prop_num: i32,
    /// New position for the prop
    pub pos: Point,
}

impl PropMoveMsg {
    /// Create a new PropMoveMsg
    pub fn new(prop_num: i32, pos: Point) -> Self {
        Self { prop_num, pos }
    }
}

impl MessagePayload for PropMoveMsg {
    fn message_id() -> MessageId {
        MessageId::PropMove
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            prop_num: buf.get_i32(),
            pos: Point::from_bytes(buf)?,
        })
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i32(self.prop_num);
        self.pos.to_bytes(buf);
    }
}

/// MessageId::PropNew - Add a new prop to the room
#[derive(Debug, Clone, PartialEq)]
pub struct PropNewMsg {
    /// Asset spec for the new prop
    pub prop_spec: AssetSpec,
    /// Initial position for the prop
    pub pos: Point,
}

impl PropNewMsg {
    /// Create a new PropNewMsg
    pub fn new(prop_spec: AssetSpec, pos: Point) -> Self {
        Self { prop_spec, pos }
    }
}

impl MessagePayload for PropNewMsg {
    fn message_id() -> MessageId {
        MessageId::PropNew
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            prop_spec: AssetSpec::from_bytes(buf)?,
            pos: Point::from_bytes(buf)?,
        })
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        self.prop_spec.to_bytes(buf);
        self.pos.to_bytes(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_list_rec() {
        let rec = RoomListRec {
            room_id: 42,
            flags: RoomFlags::PRIVATE | RoomFlags::NO_PAINTING,
            nbr_users: 5,
            name: "Test Room".to_string(),
        };

        let mut buf = vec![];
        rec.to_bytes(&mut buf);

        let parsed = RoomListRec::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed.room_id, 42);
        assert_eq!(parsed.flags, RoomFlags::PRIVATE | RoomFlags::NO_PAINTING);
        assert_eq!(parsed.nbr_users, 5);
        assert_eq!(parsed.name, "Test Room");
    }

    #[test]
    fn test_list_of_all_rooms_request() {
        let msg = ListOfAllRoomsMsg::request();
        assert!(msg.is_request());
        assert_eq!(msg.count(), 0);

        let mut buf = vec![];
        msg.to_bytes(&mut buf);
        assert_eq!(buf.len(), 0);

        let parsed = ListOfAllRoomsMsg::from_bytes(&mut &buf[..]).unwrap();
        assert!(parsed.is_request());
    }

    #[test]
    fn test_prop_del_msg() {
        let msg = PropDelMsg::new(5);

        let mut buf = vec![];
        msg.to_bytes(&mut buf);
        assert_eq!(buf.len(), 4);

        let parsed = PropDelMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed.prop_num, 5);
    }

    #[test]
    fn test_prop_del_all() {
        let msg = PropDelMsg::delete_all();
        assert_eq!(msg.prop_num, -1);
    }

    #[test]
    fn test_prop_move_msg() {
        let msg = PropMoveMsg::new(3, Point { h: 100, v: 200 });

        let mut buf = vec![];
        msg.to_bytes(&mut buf);
        assert_eq!(buf.len(), 8); // 4 + 4

        let parsed = PropMoveMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed.prop_num, 3);
        assert_eq!(parsed.pos.h, 100);
        assert_eq!(parsed.pos.v, 200);
    }

    #[test]
    fn test_prop_new_msg() {
        let msg = PropNewMsg::new(AssetSpec { id: 42, crc: 12345 }, Point { h: 150, v: 250 });

        let mut buf = vec![];
        msg.to_bytes(&mut buf);
        assert_eq!(buf.len(), 14); // 10 (AssetSpec with padding) + 4 (Point)

        let parsed = PropNewMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed.prop_spec.id, 42);
        assert_eq!(parsed.prop_spec.crc, 12345);
        assert_eq!(parsed.pos.h, 150);
        assert_eq!(parsed.pos.v, 250);
    }
}
