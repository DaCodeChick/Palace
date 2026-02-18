//! User message payloads
//!
//! This module implements message structures for user-related operations:
//! - MessageId::UserNew: New user entering a room
//! - MessageId::UserExit: User leaving a room
//! - MessageId::UserMove: User moving to a new position
//! - MessageId::UserName: User changing their name
//! - MessageId::UserColor: User changing their avatar color
//! - MessageId::UserFace: User changing their face
//! - MessageId::UserProp: User changing their props
//! - MessageId::UserDesc: Bulk user appearance change

use bytes::{Buf, BufMut};

use crate::buffer::{BufExt, BufMutExt};
use crate::messages::{MessageId, MessagePayload};
use crate::{AssetSpec, Point, RoomID, UserID};

/// UserRec - Complete user record structure
///
/// This structure is used in MessageId::UserNew and MessageId::UserList to describe
/// a user's complete appearance and state.
///
/// Size: 142 bytes
/// - userID: 4 bytes
/// - roomPos: 4 bytes (2 x i16)
/// - propSpec: 90 bytes (9 x 10 bytes, including 2-byte padding per AssetSpec)
/// - roomID: 2 bytes
/// - faceNbr: 2 bytes
/// - colorNbr: 2 bytes
/// - awayFlag: 2 bytes
/// - openToMsgs: 2 bytes
/// - nbrProps: 2 bytes
/// - name: 32 bytes (Str31)
#[derive(Debug, Clone, PartialEq)]
pub struct UserRec {
    pub user_id: UserID,
    pub room_pos: Point,
    pub prop_spec: [AssetSpec; 9],
    pub room_id: RoomID,
    pub face_nbr: i16,
    pub color_nbr: i16,
    pub away_flag: i16,
    pub open_to_msgs: i16,
    pub nbr_props: i16,
    pub name: String,
}

impl UserRec {
    /// Size of UserRec in bytes (always 142)
    pub const SIZE: usize = 4 + 4 + (9 * 10) + 2 + 2 + 2 + 2 + 2 + 2 + 32;

    /// Parse a UserRec from bytes
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        let user_id = buf.get_i32();
        let room_pos = Point {
            v: buf.get_i16(),
            h: buf.get_i16(),
        };

        // Read 9 props (always full array, even if not all used)
        let mut prop_spec = [AssetSpec::default(); 9];
        for prop in &mut prop_spec {
            *prop = AssetSpec::from_bytes(buf)?;
        }

        let room_id = buf.get_i16();
        let face_nbr = buf.get_i16();
        let color_nbr = buf.get_i16();
        let away_flag = buf.get_i16();
        let open_to_msgs = buf.get_i16();
        let nbr_props = buf.get_i16();
        let name = buf.get_str31()?;

        Ok(Self {
            user_id,
            room_pos,
            prop_spec,
            room_id,
            face_nbr,
            color_nbr,
            away_flag,
            open_to_msgs,
            nbr_props,
            name,
        })
    }

    /// Serialize this UserRec to bytes
    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i32(self.user_id);
        buf.put_i16(self.room_pos.v);
        buf.put_i16(self.room_pos.h);

        // Write all 9 props (full array)
        for prop in &self.prop_spec {
            prop.to_bytes(buf);
        }

        buf.put_i16(self.room_id);
        buf.put_i16(self.face_nbr);
        buf.put_i16(self.color_nbr);
        buf.put_i16(self.away_flag);
        buf.put_i16(self.open_to_msgs);
        buf.put_i16(self.nbr_props);
        buf.put_str31(&self.name);
    }

    /// Get the size in bytes (always 142)
    pub const fn size() -> usize {
        4 + 4 + (9 * 10) + 2 + 2 + 2 + 2 + 2 + 2 + 32
    }
}

/// MessageId::UserNew - New user entering a room
///
/// Sent from server to clients when a new user enters the room.
/// Contains a complete UserRec describing the new user.
#[derive(Debug, Clone, PartialEq)]
pub struct UserNewMsg {
    pub new_user: UserRec,
}

impl UserNewMsg {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            new_user: UserRec::from_bytes(buf)?,
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        self.new_user.to_bytes(buf);
    }
}

impl MessagePayload for UserNewMsg {
    fn message_id() -> MessageId {
        MessageId::UserNew
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Self::from_bytes(buf)
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        self.to_bytes(buf);
    }
}

/// MessageId::UserExit - User leaving a room
///
/// Sent from server to clients when a user leaves the room.
/// The UserID is in the message header's refNum field, so the payload is empty.
#[derive(Debug, Clone, PartialEq)]
pub struct UserExitMsg;

impl UserExitMsg {
    pub fn from_bytes(_buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self)
    }

    pub fn to_bytes(&self, _buf: &mut impl BufMut) {
        // Empty payload
    }
}

impl MessagePayload for UserExitMsg {
    fn message_id() -> MessageId {
        MessageId::UserExit
    }

    fn from_bytes(_buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self)
    }

    fn to_bytes(&self, _buf: &mut impl BufMut) {
        // Empty payload
    }
}

/// MessageId::UserMove - User moving to a new position
///
/// Sent bidirectionally to update a user's position in the room.
/// The UserID is in the message header's refNum field.
#[derive(Debug, Clone, PartialEq)]
pub struct UserMoveMsg {
    pub pos: Point,
}

impl UserMoveMsg {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            pos: Point {
                v: buf.get_i16(),
                h: buf.get_i16(),
            },
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i16(self.pos.v);
        buf.put_i16(self.pos.h);
    }
}

impl MessagePayload for UserMoveMsg {
    fn message_id() -> MessageId {
        MessageId::UserMove
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Self::from_bytes(buf)
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        self.to_bytes(buf);
    }
}

/// MessageId::UserName - User changing their name
///
/// Sent bidirectionally to change a user's name.
/// The UserID is in the message header's refNum field.
#[derive(Debug, Clone, PartialEq)]
pub struct UserNameMsg {
    pub name: String,
}

impl UserNameMsg {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            name: buf.get_pstring()?,
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_pstring(&self.name);
    }
}

impl MessagePayload for UserNameMsg {
    fn message_id() -> MessageId {
        MessageId::UserName
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Self::from_bytes(buf)
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        self.to_bytes(buf);
    }
}

/// MessageId::UserColor - User changing their avatar color
///
/// Sent bidirectionally to change a user's color (0-15).
/// The UserID is in the message header's refNum field.
#[derive(Debug, Clone, PartialEq)]
pub struct UserColorMsg {
    pub color_nbr: i16,
}

impl UserColorMsg {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            color_nbr: buf.get_i16(),
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i16(self.color_nbr);
    }
}

impl MessagePayload for UserColorMsg {
    fn message_id() -> MessageId {
        MessageId::UserColor
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Self::from_bytes(buf)
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        self.to_bytes(buf);
    }
}

/// MessageId::UserFace - User changing their face
///
/// Sent bidirectionally to change a user's face (0-15).
/// The UserID is in the message header's refNum field.
#[derive(Debug, Clone, PartialEq)]
pub struct UserFaceMsg {
    pub face_nbr: i16,
}

impl UserFaceMsg {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            face_nbr: buf.get_i16(),
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i16(self.face_nbr);
    }
}

impl MessagePayload for UserFaceMsg {
    fn message_id() -> MessageId {
        MessageId::UserFace
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Self::from_bytes(buf)
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        self.to_bytes(buf);
    }
}

/// MessageId::UserProp - User changing their props
///
/// Sent bidirectionally to change a user's props (0-9 props).
/// The UserID is in the message header's refNum field.
#[derive(Debug, Clone, PartialEq)]
pub struct UserPropMsg {
    pub props: Vec<AssetSpec>,
}

impl UserPropMsg {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        let nbr_props = buf.get_i32();
        let mut props = Vec::with_capacity(nbr_props as usize);

        for _ in 0..nbr_props {
            props.push(AssetSpec::from_bytes(buf)?);
        }

        Ok(Self { props })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i32(self.props.len() as i32);
        for prop in &self.props {
            prop.to_bytes(buf);
        }
    }
}

impl MessagePayload for UserPropMsg {
    fn message_id() -> MessageId {
        MessageId::UserProp
    }

    fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Self::from_bytes(buf)
    }

    fn to_bytes(&self, buf: &mut impl BufMut) {
        self.to_bytes(buf);
    }
}

/// MessageId::UserDesc - Bulk user appearance change
///
/// Sent bidirectionally to change face, color, and props all at once.
/// The UserID is in the message header's refNum field.
#[derive(Debug, Clone, PartialEq)]
pub struct UserDescMsg {
    pub face_nbr: i16,
    pub color_nbr: i16,
    pub props: Vec<AssetSpec>,
}

impl UserDescMsg {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        let face_nbr = buf.get_i16();
        let color_nbr = buf.get_i16();
        let nbr_props = buf.get_i32();

        let mut props = Vec::with_capacity(nbr_props as usize);
        for _ in 0..nbr_props {
            props.push(AssetSpec::from_bytes(buf)?);
        }

        Ok(Self {
            face_nbr,
            color_nbr,
            props,
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i16(self.face_nbr);
        buf.put_i16(self.color_nbr);
        buf.put_i32(self.props.len() as i32);
        for prop in &self.props {
            prop.to_bytes(buf);
        }
    }
}

impl MessagePayload for UserDescMsg {
    fn message_id() -> MessageId {
        MessageId::UserDesc
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
    use bytes::BytesMut;

    #[test]
    fn test_user_rec_roundtrip() {
        let user = UserRec {
            user_id: 12345,
            room_pos: Point { v: 100, h: 200 },
            prop_spec: [AssetSpec::default(); 9],
            room_id: 42,
            face_nbr: 3,
            color_nbr: 7,
            away_flag: 0,
            open_to_msgs: 1,
            nbr_props: 0,
            name: "TestUser".to_string(),
        };

        let mut buf = BytesMut::new();
        user.to_bytes(&mut buf);

        assert_eq!(buf.len(), UserRec::size());

        let mut reader = buf.freeze();
        let parsed = UserRec::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, user);
    }

    #[test]
    fn test_user_new_msg_roundtrip() {
        let msg = UserNewMsg {
            new_user: UserRec {
                user_id: 999,
                room_pos: Point { v: 50, h: 75 },
                prop_spec: [AssetSpec::default(); 9],
                room_id: 10,
                face_nbr: 1,
                color_nbr: 2,
                away_flag: 0,
                open_to_msgs: 1,
                nbr_props: 0,
                name: "NewUser".to_string(),
            },
        };

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        let mut reader = buf.freeze();
        let parsed = UserNewMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_user_new_msg_payload_trait() {
        let msg = UserNewMsg {
            new_user: UserRec {
                user_id: 999,
                room_pos: Point { v: 50, h: 75 },
                prop_spec: [AssetSpec::default(); 9],
                room_id: 10,
                face_nbr: 1,
                color_nbr: 2,
                away_flag: 0,
                open_to_msgs: 1,
                nbr_props: 0,
                name: "NewUser".to_string(),
            },
        };

        // Test to_message()
        let message = msg.to_message(0);
        assert_eq!(message.msg_id, MessageId::UserNew);
        assert_eq!(message.ref_num, 0);

        // Test parse_payload()
        let parsed = message.parse_payload::<UserNewMsg>().unwrap();
        assert_eq!(parsed.new_user, msg.new_user);
    }

    #[test]
    fn test_user_exit_msg() {
        let msg = UserExitMsg;

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        assert_eq!(buf.len(), 0); // Empty payload

        let mut reader = buf.freeze();
        let parsed = UserExitMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_user_move_msg_roundtrip() {
        let msg = UserMoveMsg {
            pos: Point { v: 123, h: 456 },
        };

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        assert_eq!(buf.len(), 4); // 2 x i16

        let mut reader = buf.freeze();
        let parsed = UserMoveMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_user_name_msg_roundtrip() {
        let msg = UserNameMsg {
            name: "Alice".to_string(),
        };

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        let mut reader = buf.freeze();
        let parsed = UserNameMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_user_color_msg_roundtrip() {
        let msg = UserColorMsg { color_nbr: 5 };

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        assert_eq!(buf.len(), 2);

        let mut reader = buf.freeze();
        let parsed = UserColorMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_user_face_msg_roundtrip() {
        let msg = UserFaceMsg { face_nbr: 12 };

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        assert_eq!(buf.len(), 2);

        let mut reader = buf.freeze();
        let parsed = UserFaceMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_user_prop_msg_roundtrip() {
        let msg = UserPropMsg {
            props: vec![
                AssetSpec {
                    id: 100,
                    crc: 0x12345678,
                },
                AssetSpec {
                    id: 200,
                    crc: 0x87654321,
                },
            ],
        };

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        let mut reader = buf.freeze();
        let parsed = UserPropMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_user_desc_msg_roundtrip() {
        let msg = UserDescMsg {
            face_nbr: 8,
            color_nbr: 4,
            props: vec![AssetSpec {
                id: 300,
                crc: 0xABCDEF00,
            }],
        };

        let mut buf = BytesMut::new();
        msg.to_bytes(&mut buf);

        let mut reader = buf.freeze();
        let parsed = UserDescMsg::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, msg);
    }
}
