//! Hotspot operation messages
//!
//! This module contains messages for hotspot operations:
//! - SpotDelMsg: Delete a hotspot from the room
//! - SpotMoveMsg: Move a hotspot to a new position
//! - SpotNewMsg: Create a new hotspot in the room
//! - SpotStateMsg: Change the state of a hotspot

use bytes::{Buf, BufMut};

use crate::messages::{MessageId, MessagePayload};
use crate::Point;

/// MessageId::SpotDel - Delete a hotspot from the room
///
/// Client requests server to delete a hotspot. If successful,
/// server replaces the room with a new room lacking the hotspot.
#[derive(Debug, Clone, PartialEq)]
pub struct SpotDelMsg {
    /// ID of the hotspot to delete
    pub spot_id: i32,
}

impl SpotDelMsg {
    /// Create a new SpotDelMsg
    pub const fn new(spot_id: i32) -> Self {
        Self { spot_id }
    }
}

impl MessagePayload for SpotDelMsg {
    fn message_id() -> MessageId {
        MessageId::SpotDel
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            spot_id: buf.get_i32(),
        })
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i32(self.spot_id);
    }
}

/// MessageId::SpotMove - Move a hotspot to a new position
///
/// Used to modify the screen location of a hotspot.
#[derive(Debug, Clone, PartialEq)]
pub struct SpotMoveMsg {
    /// Room ID containing the hotspot
    pub room_id: i16,
    /// Hotspot ID to move
    pub spot_id: i32,
    /// New position for the hotspot
    pub pos: Point,
}

impl SpotMoveMsg {
    /// Create a new SpotMoveMsg
    pub const fn new(room_id: i16, spot_id: i32, pos: Point) -> Self {
        Self {
            room_id,
            spot_id,
            pos,
        }
    }
}

impl MessagePayload for SpotMoveMsg {
    fn message_id() -> MessageId {
        MessageId::SpotMove
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            room_id: buf.get_i16(),
            spot_id: buf.get_i32(),
            pos: Point::from_bytes(buf)?,
        })
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i16(self.room_id);
        buf.put_i32(self.spot_id);
        self.pos.to_bytes(buf);
    }
}

/// MessageId::SpotNew - Create a new hotspot in the room
///
/// Client requests server to create a hotspot with default configuration.
/// If successful, server replaces room with new room containing the hotspot.
/// Empty payload.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SpotNewMsg;

impl MessagePayload for SpotNewMsg {
    fn message_id() -> MessageId {
        MessageId::SpotNew
    }

    fn from_bytes(_buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self)
    }

    fn to_bytes(&self, _buf: &mut impl BufMut) {}
}

/// MessageId::SpotState - Change the state of a hotspot
///
/// Used to modify the state field of a hotspot (for stateful hotspots).
#[derive(Debug, Clone, PartialEq)]
pub struct SpotStateMsg {
    /// Room ID containing the hotspot
    pub room_id: i16,
    /// Hotspot ID to modify
    pub spot_id: i32,
    /// New state value
    pub state: i16,
}

impl SpotStateMsg {
    /// Create a new SpotStateMsg
    pub const fn new(room_id: i16, spot_id: i32, state: i16) -> Self {
        Self {
            room_id,
            spot_id,
            state,
        }
    }
}

impl MessagePayload for SpotStateMsg {
    fn message_id() -> MessageId {
        MessageId::SpotState
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            room_id: buf.get_i16(),
            spot_id: buf.get_i32(),
            state: buf.get_i16(),
        })
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i16(self.room_id);
        buf.put_i32(self.spot_id);
        buf.put_i16(self.state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spot_del_msg() {
        let msg = SpotDelMsg::new(42);

        let mut buf = vec![];
        msg.to_bytes(&mut buf);
        assert_eq!(buf.len(), 4);

        let parsed = SpotDelMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed.spot_id, 42);
    }

    #[test]
    fn test_spot_move_msg() {
        let msg = SpotMoveMsg::new(5, 10, Point { h: 100, v: 200 });

        let mut buf = vec![];
        msg.to_bytes(&mut buf);
        assert_eq!(buf.len(), 10); // 2 + 4 + 4

        let parsed = SpotMoveMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed.room_id, 5);
        assert_eq!(parsed.spot_id, 10);
        assert_eq!(parsed.pos.h, 100);
        assert_eq!(parsed.pos.v, 200);
    }

    #[test]
    fn test_spot_new_msg() {
        let msg = SpotNewMsg;

        let mut buf = vec![];
        msg.to_bytes(&mut buf);
        assert_eq!(buf.len(), 0);

        let parsed = SpotNewMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_spot_state_msg() {
        let msg = SpotStateMsg::new(7, 15, 3);

        let mut buf = vec![];
        msg.to_bytes(&mut buf);
        assert_eq!(buf.len(), 8); // 2 + 4 + 2

        let parsed = SpotStateMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed.room_id, 7);
        assert_eq!(parsed.spot_id, 15);
        assert_eq!(parsed.state, 3);
    }
}
