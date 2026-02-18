//! Door operation messages
//!
//! This module contains messages for door operations:
//! - DoorLockMsg: Lock a door
//! - DoorUnlockMsg: Unlock a door

use bytes::{Buf, BufMut};

use crate::messages::{MessageId, MessagePayload};

/// MessageId::DoorLock
///
/// Client-to-server: Request to lock a door
/// Server-to-clients: Notification that a door was locked
///
/// Contains:
/// - room_id: RoomID of the room containing the door
/// - door_id: HotspotID of the door hotspot
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoorLockMsg {
    pub room_id: i16,
    pub door_id: i32,
}

impl DoorLockMsg {
    /// Create a new DoorLockMsg
    pub const fn new(room_id: i16, door_id: i32) -> Self {
        Self { room_id, door_id }
    }
}

impl MessagePayload for DoorLockMsg {
    fn message_id() -> MessageId {
        MessageId::DoorLock
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            room_id: buf.get_i16(),
            door_id: buf.get_i32(),
        })
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i16(self.room_id);
        buf.put_i32(self.door_id);
    }
}

/// MessageId::DoorUnlock
///
/// Client-to-server: Request to unlock a door
/// Server-to-clients: Notification that a door was unlocked
///
/// Contains:
/// - room_id: RoomID of the room containing the door
/// - door_id: HotspotID of the door hotspot
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoorUnlockMsg {
    pub room_id: i16,
    pub door_id: i32,
}

impl DoorUnlockMsg {
    /// Create a new DoorUnlockMsg
    pub const fn new(room_id: i16, door_id: i32) -> Self {
        Self { room_id, door_id }
    }
}

impl MessagePayload for DoorUnlockMsg {
    fn message_id() -> MessageId {
        MessageId::DoorUnlock
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            room_id: buf.get_i16(),
            door_id: buf.get_i32(),
        })
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i16(self.room_id);
        buf.put_i32(self.door_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_door_lock_msg() {
        let msg = DoorLockMsg::new(10, 42);

        let mut buf = vec![];
        msg.to_bytes(&mut buf);
        assert_eq!(buf.len(), 6); // 2 + 4

        let parsed = DoorLockMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed.room_id, 10);
        assert_eq!(parsed.door_id, 42);
    }

    #[test]
    fn test_door_unlock_msg() {
        let msg = DoorUnlockMsg::new(15, 88);

        let mut buf = vec![];
        msg.to_bytes(&mut buf);
        assert_eq!(buf.len(), 6); // 2 + 4

        let parsed = DoorUnlockMsg::from_bytes(&mut &buf[..]).unwrap();
        assert_eq!(parsed.room_id, 15);
        assert_eq!(parsed.door_id, 88);
    }
}
