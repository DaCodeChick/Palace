//! Picture operation messages
//!
//! This module contains messages for picture operations:
//! - PictMoveMsg: Move a picture layer

use bytes::{Buf, BufMut};

use crate::messages::{MessageId, MessagePayload};
use crate::Point;

/// MessageId::PictMove
///
/// Client-to-server: Request to move a picture layer
/// Server-to-clients: Notification that a picture was moved
///
/// Contains:
/// - room_id: RoomID of the room containing the picture
/// - spot_id: HotspotID of the picture itself
/// - pos: New position for the picture
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PictMoveMsg {
    pub room_id: i16,
    pub spot_id: i32,
    pub pos: Point,
}

impl PictMoveMsg {
    /// Create a new PictMoveMsg
    pub fn new(room_id: i16, spot_id: i32, pos: Point) -> Self {
        Self {
            room_id,
            spot_id,
            pos,
        }
    }
}

impl MessagePayload for PictMoveMsg {
    fn message_id() -> MessageId {
        MessageId::PictMove
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pict_move_msg() {
        let msg = PictMoveMsg::new(20, 100, Point { h: 300, v: 400 });

        let mut buf = vec![];
        msg.to_bytes(&mut buf);
        assert_eq!(buf.len(), 10); // 2 + 4 + 4

        let parsed = PictMoveMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed.room_id, 20);
        assert_eq!(parsed.spot_id, 100);
        assert_eq!(parsed.pos.h, 300);
        assert_eq!(parsed.pos.v, 400);
    }
}
