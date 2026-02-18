//! Room record structures
//!
//! This module contains the core data structures for room records:
//! - LPropRec: Loose prop record
//! - PictureRec: Picture layer record
//! - Hotspot: Interactive hotspot record
//! - RoomRec: Complete room description

use bytes::{Buf, BufMut, Bytes};

use crate::buffer::BufExt;
use crate::messages::flags::RoomFlags;
use crate::room::{HotspotState, HotspotType};
use crate::{AssetSpec, Point};

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
        use crate::messages::flags::RoomFlags;

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
}
