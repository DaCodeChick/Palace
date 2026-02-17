//! Room message payloads
//!
//! This module implements message structures for room-related operations:
//! - MSG_ROOMGOTO: Client requests to move to a different room
//! - MSG_ROOMDESC: Server describes a room (complex structure with RoomRec)
//! - MSG_ROOMDESCEND: Marks end of room description sequence
//! - MSG_ROOMNEW: Create a new room
//! - MSG_ROOMSETDESC: Update room description
//!
//! RoomRec is a complex structure with variable-length data including hotspots,
//! pictures, loose props, draw commands, and embedded strings.

use bytes::{Buf, BufMut, Bytes};

use crate::buffer::BufExt;
use crate::messages::{flags::RoomFlags, MessageId, MessagePayload};
use crate::room::{HotspotState, HotspotType};
use crate::{AssetSpec, Point, RoomID};

// ============================================================================
// Room Record Structures
// ============================================================================

/// Linked list record - used internally in various structures.
///
/// This structure is only used internally in the Palace client and its
/// contents are ignored when transmitted over the network.
///
/// Size: 4 bytes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LLRec {
    pub next_ofst: i16,
    pub reserved: i16,
}

impl LLRec {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            next_ofst: buf.get_i16(),
            reserved: buf.get_i16(),
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i16(self.next_ofst);
        buf.put_i16(self.reserved);
    }
}

/// Loose prop record - describes a prop in the room.
///
/// Size: 24 bytes (4 + 8 + 4 + 4 + 4)
#[derive(Debug, Clone, PartialEq)]
pub struct LPropRec {
    /// Linked list link (used internally, ignored over wire)
    pub link: LLRec,
    /// Asset identifier for the prop
    pub prop_spec: AssetSpec,
    /// Prop behavior flags
    pub flags: i32,
    /// Arbitrary use variable (used by client)
    pub ref_con: i32,
    /// Screen location for prop display
    pub loc: Point,
}

impl LPropRec {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            link: LLRec::from_bytes(buf)?,
            prop_spec: AssetSpec::from_bytes(buf)?,
            flags: buf.get_i32(),
            ref_con: buf.get_i32(),
            loc: Point::from_bytes(buf)?,
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        self.link.to_bytes(buf);
        self.prop_spec.to_bytes(buf);
        buf.put_i32(self.flags);
        buf.put_i32(self.ref_con);
        self.loc.to_bytes(buf);
    }
}

/// Picture record - describes a picture layer in the room.
///
/// Size: 12 bytes (4 + 2 + 2 + 2 + 2)
#[derive(Debug, Clone, PartialEq)]
pub struct PictureRec {
    /// Arbitrary use variable (not used)
    pub ref_con: i32,
    /// Picture ID number
    pub pic_id: i16,
    /// Offset into varBuf for picture name (PString)
    pub pic_name_ofst: i16,
    /// Transparent color value
    pub trans_color: i16,
    /// Reserved for alignment (should be 0)
    pub reserved: i16,
}

impl PictureRec {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        Ok(Self {
            ref_con: buf.get_i32(),
            pic_id: buf.get_i16(),
            pic_name_ofst: buf.get_i16(),
            trans_color: buf.get_i16(),
            reserved: buf.get_i16(),
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i32(self.ref_con);
        buf.put_i16(self.pic_id);
        buf.put_i16(self.pic_name_ofst);
        buf.put_i16(self.trans_color);
        buf.put_i16(self.reserved);
    }
}

/// Hotspot structure - describes a clickable interactive area in a room.
///
/// Hotspots can trigger scripts, link to other rooms, or control access.
/// They contain variable-length data (polygon points, scripts, states)
/// stored in the RoomRec's varBuf.
///
/// Size: 48 bytes (fixed part)
/// Calculation: 4+4+4+4+4+2+2+2+2+2+2+2+2+2+2+2+2+2+2 = 48 bytes
#[derive(Debug, Clone, PartialEq)]
pub struct Hotspot {
    /// Bitmask of script events this hotspot responds to
    pub script_event_mask: i32,
    /// Hotspot behavior flags
    pub flags: i32,
    /// Security information
    pub secure_info: i32,
    /// Arbitrary use variable
    pub ref_con: i32,
    /// Location of hotspot
    pub loc: Point,
    /// Hotspot ID number
    pub id: i16,
    /// Destination room ID (for door-type hotspots)
    pub dest: i16,
    /// Number of polygon points
    pub nbr_pts: i16,
    /// Offset into varBuf for polygon points array
    pub pts_ofst: i16,
    /// Hotspot type (door, bolt, normal, etc.)
    pub hotspot_type: HotspotType,
    /// Group ID for related hotspots
    pub group_id: i16,
    /// Number of scripts
    pub nbr_scripts: i16,
    /// Offset into varBuf for script records
    pub script_rec_ofst: i16,
    /// Current state (locked/unlocked)
    pub state: HotspotState,
    /// Number of states
    pub nbr_states: i16,
    /// Offset into varBuf for state records
    pub state_rec_ofst: i16,
    /// Offset into varBuf for hotspot name (PString)
    pub name_ofst: i16,
    /// Offset into varBuf for script text (PString)
    pub script_text_ofst: i16,
    /// Reserved for alignment
    pub align_reserved: i16,
}

impl Hotspot {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        let script_event_mask = buf.get_i32();
        let flags = buf.get_i32();
        let secure_info = buf.get_i32();
        let ref_con = buf.get_i32();
        let loc = Point::from_bytes(buf)?;
        let id = buf.get_i16();
        let dest = buf.get_i16();
        let nbr_pts = buf.get_i16();
        let pts_ofst = buf.get_i16();
        let type_raw = buf.get_i16();
        let group_id = buf.get_i16();
        let nbr_scripts = buf.get_i16();
        let script_rec_ofst = buf.get_i16();
        let state_raw = buf.get_i16();
        let nbr_states = buf.get_i16();
        let state_rec_ofst = buf.get_i16();
        let name_ofst = buf.get_i16();
        let script_text_ofst = buf.get_i16();
        let align_reserved = buf.get_i16();

        let hotspot_type = HotspotType::from_i16(type_raw).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid hotspot type: {}", type_raw),
            )
        })?;

        let state = HotspotState::from_i16(state_raw).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid hotspot state: {}", state_raw),
            )
        })?;

        Ok(Self {
            script_event_mask,
            flags,
            secure_info,
            ref_con,
            loc,
            id,
            dest,
            nbr_pts,
            pts_ofst,
            hotspot_type,
            group_id,
            nbr_scripts,
            script_rec_ofst,
            state,
            nbr_states,
            state_rec_ofst,
            name_ofst,
            script_text_ofst,
            align_reserved,
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i32(self.script_event_mask);
        buf.put_i32(self.flags);
        buf.put_i32(self.secure_info);
        buf.put_i32(self.ref_con);
        self.loc.to_bytes(buf);
        buf.put_i16(self.id);
        buf.put_i16(self.dest);
        buf.put_i16(self.nbr_pts);
        buf.put_i16(self.pts_ofst);
        buf.put_i16(self.hotspot_type.as_i16());
        buf.put_i16(self.group_id);
        buf.put_i16(self.nbr_scripts);
        buf.put_i16(self.script_rec_ofst);
        buf.put_i16(self.state.as_i16());
        buf.put_i16(self.nbr_states);
        buf.put_i16(self.state_rec_ofst);
        buf.put_i16(self.name_ofst);
        buf.put_i16(self.script_text_ofst);
        buf.put_i16(self.align_reserved);
    }
}

/// Room record - complete description of a Palace room.
///
/// This is a complex structure with variable-length data including:
/// - Room metadata (name, background picture, artist, password)
/// - Hotspots (interactive areas with scripts)
/// - Pictures (layered background images)
/// - Draw commands (vector graphics)
/// - Loose props (decorative objects)
///
/// The varBuf field contains all variable-length data, with offsets
/// stored in the fixed-size fields. Arrays must be 4-byte aligned.
///
/// Size: 40 bytes (fixed) + lenVars bytes (variable)
/// Calculation: 4 (room_flags) + 4 (faces_id) + 2×16 (sixteen i16 fields) = 8 + 32 = 40 bytes
#[derive(Debug, Clone, PartialEq)]
pub struct RoomRec {
    /// Room attribute flags
    pub room_flags: RoomFlags,
    /// Default avatar face ID for users in this room
    pub faces_id: i32,
    /// Room ID number
    pub room_id: RoomID,
    /// Offset into varBuf for room name (PString)
    pub room_name_ofst: i16,
    /// Offset into varBuf for background picture name (PString)
    pub pict_name_ofst: i16,
    /// Offset into varBuf for artist name (PString)
    pub artist_name_ofst: i16,
    /// Offset into varBuf for room password (PString)
    pub password_ofst: i16,
    /// Number of hotspots in room
    pub nbr_hotspots: i16,
    /// Offset into varBuf for hotspots array (4-byte aligned)
    pub hotspot_ofst: i16,
    /// Number of pictures in room
    pub nbr_pictures: i16,
    /// Offset into varBuf for pictures array (4-byte aligned)
    pub picture_ofst: i16,
    /// Number of draw commands
    pub nbr_draw_cmds: i16,
    /// Offset into varBuf for first draw command (4-byte aligned)
    pub first_draw_cmd: i16,
    /// Number of users currently in room
    pub nbr_people: i16,
    /// Number of loose props in room
    pub nbr_lprops: i16,
    /// Offset into varBuf for loose props array (4-byte aligned)
    pub first_lprop: i16,
    /// Reserved for alignment (should be 0)
    pub reserved: i16,
    /// Length of variable data buffer
    pub len_vars: i16,
    /// Variable-length data buffer
    pub var_buf: Bytes,
}

impl RoomRec {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        let room_flags_raw = buf.get_i32();
        let faces_id = buf.get_i32();
        let room_id = buf.get_i16();
        let room_name_ofst = buf.get_i16();
        let pict_name_ofst = buf.get_i16();
        let artist_name_ofst = buf.get_i16();
        let password_ofst = buf.get_i16();
        let nbr_hotspots = buf.get_i16();
        let hotspot_ofst = buf.get_i16();
        let nbr_pictures = buf.get_i16();
        let picture_ofst = buf.get_i16();
        let nbr_draw_cmds = buf.get_i16();
        let first_draw_cmd = buf.get_i16();
        let nbr_people = buf.get_i16();
        let nbr_lprops = buf.get_i16();
        let first_lprop = buf.get_i16();
        let reserved = buf.get_i16();
        let len_vars = buf.get_i16();

        let room_flags = RoomFlags::from_bits_truncate(room_flags_raw as u16);

        // Read variable buffer
        let var_buf = if len_vars > 0 {
            buf.copy_to_bytes(len_vars as usize)
        } else {
            Bytes::new()
        };

        Ok(Self {
            room_flags,
            faces_id,
            room_id,
            room_name_ofst,
            pict_name_ofst,
            artist_name_ofst,
            password_ofst,
            nbr_hotspots,
            hotspot_ofst,
            nbr_pictures,
            picture_ofst,
            nbr_draw_cmds,
            first_draw_cmd,
            nbr_people,
            nbr_lprops,
            first_lprop,
            reserved,
            len_vars,
            var_buf,
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i32(self.room_flags.bits() as i32);
        buf.put_i32(self.faces_id);
        buf.put_i16(self.room_id);
        buf.put_i16(self.room_name_ofst);
        buf.put_i16(self.pict_name_ofst);
        buf.put_i16(self.artist_name_ofst);
        buf.put_i16(self.password_ofst);
        buf.put_i16(self.nbr_hotspots);
        buf.put_i16(self.hotspot_ofst);
        buf.put_i16(self.nbr_pictures);
        buf.put_i16(self.picture_ofst);
        buf.put_i16(self.nbr_draw_cmds);
        buf.put_i16(self.first_draw_cmd);
        buf.put_i16(self.nbr_people);
        buf.put_i16(self.nbr_lprops);
        buf.put_i16(self.first_lprop);
        buf.put_i16(self.reserved);
        buf.put_i16(self.len_vars);
        buf.put_slice(&self.var_buf);
    }

    /// Get room name from varBuf
    pub fn room_name(&self) -> std::io::Result<String> {
        self.get_pstring(self.room_name_ofst)
    }

    /// Get background picture name from varBuf
    pub fn pict_name(&self) -> std::io::Result<String> {
        self.get_pstring(self.pict_name_ofst)
    }

    /// Get artist name from varBuf
    pub fn artist_name(&self) -> std::io::Result<String> {
        self.get_pstring(self.artist_name_ofst)
    }

    /// Get room password from varBuf
    pub fn password(&self) -> std::io::Result<String> {
        self.get_pstring(self.password_ofst)
    }

    /// Helper to extract PString from varBuf at given offset
    fn get_pstring(&self, offset: i16) -> std::io::Result<String> {
        if offset < 0 || offset as usize >= self.var_buf.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid offset: {}", offset),
            ));
        }

        let mut buf = &self.var_buf[offset as usize..];
        buf.get_pstring()
    }
}

// ============================================================================
// Room Messages
// ============================================================================

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

/// MSG_ROOMDESC - Server describes a room to the client
///
/// This is one of the most complex messages in the Palace protocol.
/// It contains complete room information including metadata, hotspots,
/// pictures, loose props, and draw commands.
///
/// Sent from server to client when:
/// - User enters a new room (after MSG_ROOMGOTO)
/// - Room description is updated (after MSG_ROOMSETDESC)
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
    use bytes::BytesMut;

    #[test]
    fn test_llrec_roundtrip() {
        let rec = LLRec {
            next_ofst: 100,
            reserved: 0,
        };

        let mut buf = BytesMut::new();
        rec.to_bytes(&mut buf);

        assert_eq!(buf.len(), 4);

        let mut reader = buf.freeze();
        let parsed = LLRec::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, rec);
    }

    #[test]
    fn test_lprop_rec_roundtrip() {
        let rec = LPropRec {
            link: LLRec {
                next_ofst: 0,
                reserved: 0,
            },
            prop_spec: AssetSpec {
                id: 12345,
                crc: 0xABCDEF01,
            },
            flags: 0x00100000, // 20-bit format
            ref_con: 42,
            loc: Point { v: 100, h: 200 },
        };

        let mut buf = BytesMut::new();
        rec.to_bytes(&mut buf);

        assert_eq!(buf.len(), 24); // 4 + 8 + 4 + 4 + 4

        let mut reader = buf.freeze();
        let parsed = LPropRec::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, rec);
    }

    #[test]
    fn test_picture_rec_roundtrip() {
        let rec = PictureRec {
            ref_con: 0,
            pic_id: 100,
            pic_name_ofst: 10,
            trans_color: -1, // 0xFFFF as i16 = -1
            reserved: 0,
        };

        let mut buf = BytesMut::new();
        rec.to_bytes(&mut buf);

        assert_eq!(buf.len(), 12);

        let mut reader = buf.freeze();
        let parsed = PictureRec::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, rec);
    }

    #[test]
    fn test_hotspot_roundtrip() {
        let hotspot = Hotspot {
            script_event_mask: 0x0007, // SELECT | ENTER | LEAVE
            flags: 0,
            secure_info: 0,
            ref_con: 0,
            loc: Point { v: 100, h: 200 },
            id: 1,
            dest: 10,
            nbr_pts: 4,
            pts_ofst: 0,
            hotspot_type: HotspotType::Door,
            group_id: 0,
            nbr_scripts: 1,
            script_rec_ofst: 100,
            state: HotspotState::Unlocked,
            nbr_states: 0,
            state_rec_ofst: 0,
            name_ofst: 50,
            script_text_ofst: 150,
            align_reserved: 0,
        };

        let mut buf = BytesMut::new();
        hotspot.to_bytes(&mut buf);

        assert_eq!(buf.len(), 48);

        let mut reader = buf.freeze();
        let parsed = Hotspot::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, hotspot);
    }

    #[test]
    fn test_room_rec_roundtrip() {
        // Create a simple room with a name in varBuf
        let room_name = "Test Room";
        let mut var_buf = BytesMut::new();
        var_buf.put_u8(room_name.len() as u8); // PString length
        var_buf.put_slice(room_name.as_bytes()); // PString data

        let room = RoomRec {
            room_flags: RoomFlags::NO_PAINTING | RoomFlags::DROP_ZONE,
            faces_id: 0,
            room_id: 42,
            room_name_ofst: 0, // Points to start of varBuf
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
            reserved: 0,
            len_vars: var_buf.len() as i16,
            var_buf: var_buf.freeze(),
        };

        let mut buf = BytesMut::new();
        room.to_bytes(&mut buf);

        // Fixed part size calculation:
        // 4 (room_flags i32) + 4 (faces_id) + 2×16 (sixteen i16 fields) = 8 + 32 = 40 bytes
        // Variable part: room.len_vars
        let expected_size = 40 + room.len_vars as usize;
        assert_eq!(buf.len(), expected_size);

        let mut reader = buf.freeze();
        let parsed = RoomRec::from_bytes(&mut reader).unwrap();

        assert_eq!(parsed, room);
        assert_eq!(parsed.room_name().unwrap(), room_name);
    }

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
            reserved: 0,
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
