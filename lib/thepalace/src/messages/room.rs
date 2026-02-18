//! Room message payloads
//!
//! This module implements message structures for room-related operations:
//! - MessageId::RoomGoto: Client requests to move to a different room
//! - MessageId::RoomDesc: Server describes a room (complex structure with RoomRec)
//! - MessageId::RoomDescEnd: Marks end of room description sequence
//! - MessageId::RoomNew: Create a new room
//! - MessageId::RoomSetDesc: Update room description
//!
//! RoomRec is a complex structure with variable-length data including hotspots,
//! pictures, loose props, draw commands, and embedded strings.

use bytes::{Buf, BufMut, Bytes};

use crate::buffer::{BufExt, BufMutExt};
use crate::messages::{flags::RoomFlags, MessageId, MessagePayload};
use crate::room::{HotspotState, HotspotType};
use crate::{AssetSpec, Point};

// ============================================================================
// Room Record Structures
// ============================================================================

/// Loose prop record - describes a prop in the room.
///
/// Size: 24 bytes (4 padding + 8 + 4 + 4 + 4)
#[derive(Debug, Clone, PartialEq)]
pub struct LPropRec {
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
        // Skip 4 bytes of padding (originally a linked list pointer for client use)
        let _ = buf.get_i32();

        Ok(Self {
            prop_spec: AssetSpec::from_bytes(buf)?,
            flags: buf.get_i32(),
            ref_con: buf.get_i32(),
            loc: Point::from_bytes(buf)?,
        })
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        // Write 4 bytes of zero padding (originally a linked list pointer for client use)
        buf.put_i32(0);

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
}

impl PictureRec {
    pub fn from_bytes(buf: &mut impl Buf) -> std::io::Result<Self> {
        let rec = Self {
            ref_con: buf.get_i32(),
            pic_id: buf.get_i16(),
            pic_name_ofst: buf.get_i16(),
            trans_color: buf.get_i16(),
        };
        // Skip 2 bytes of padding
        let _ = buf.get_i16();
        Ok(rec)
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_i32(self.ref_con);
        buf.put_i16(self.pic_id);
        buf.put_i16(self.pic_name_ofst);
        buf.put_i16(self.trans_color);
        // Write 2 bytes of zero padding
        buf.put_i16(0);
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
        // Skip 2 bytes of padding
        let _ = buf.get_i16();

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
        // Write 2 bytes of zero padding
        buf.put_i16(0);
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
    pub room_id: i16,
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
        // Skip 2 bytes of padding
        let _ = buf.get_i16();
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
        // Write 2 bytes of zero padding
        buf.put_i16(0);
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
    use bytes::BytesMut;

    #[test]
    fn test_lprop_rec_roundtrip() {
        let rec = LPropRec {
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

        assert_eq!(buf.len(), 26); // 4 padding + 10 (AssetSpec with padding) + 4 + 4 + 4

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

// ============================================================================
// Room List & Prop Operation Messages
// ============================================================================

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
mod prop_tests {
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

// ============================================================================
// Hotspot Operation Messages
// ============================================================================

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
    pub fn new(spot_id: i32) -> Self {
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
    pub fn new(room_id: i16, spot_id: i32, pos: Point) -> Self {
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
    pub fn new(room_id: i16, spot_id: i32, state: i16) -> Self {
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

// ============================================================================
// Door Operation Messages
// ============================================================================

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
    pub fn new(room_id: i16, door_id: i32) -> Self {
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
    pub fn new(room_id: i16, door_id: i32) -> Self {
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

// ============================================================================
// Picture Operation Messages
// ============================================================================

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
mod door_picture_tests {
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

#[cfg(test)]
mod hotspot_tests {
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
