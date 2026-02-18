//! User data structures

use bytes::{Buf, BufMut};

use crate::buffer::{BufExt, BufMutExt};
use crate::{AssetSpec, Point};

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
    pub user_id: i32,
    pub room_pos: Point,
    pub prop_spec: [AssetSpec; 9],
    pub room_id: i16,
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
}
