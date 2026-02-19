//! Room script to protocol converter.
//!
//! Converts parsed room script AST structures (`RoomDecl`) into Palace protocol
//! structures (`RoomRec`) that can be used as room templates.
//!
//! **Important:** The converter produces room **templates** with static data only.
//! Runtime fields (nbr_people, nbr_lprops, nbr_draw_cmds) are set to zero.

use bytes::{BufMut, Bytes, BytesMut};

use crate::iptscrae::{EventMask, RoomDecl, Script};
use crate::messages::room::{Hotspot, PictureRec, RoomRec};
use crate::room::{HotspotState, HotspotType};
use crate::Point;

/// Errors that can occur during room script conversion.
#[derive(Debug, Clone)]
pub enum ConversionError {
    /// varBuf would exceed i16::MAX (32767 bytes)
    VarBufTooLarge { size: usize },

    /// Too many hotspots (max i16::MAX)
    TooManyHotspots { count: usize },

    /// Too many pictures (max i16::MAX)
    TooManyPictures { count: usize },

    /// Too many points in outline (max i16::MAX)
    TooManyPoints { hotspot_id: i16, count: usize },

    /// Too many states (max i16::MAX)
    TooManyStates { hotspot_id: i16, count: usize },

    /// String too long for PString (max 255 bytes)
    StringTooLong { field: String, length: usize },

    /// Script serialization failed
    ScriptSerializationError { message: String },
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::VarBufTooLarge { size } => {
                write!(f, "varBuf too large: {} bytes (max 32767)", size)
            }
            ConversionError::TooManyHotspots { count } => {
                write!(f, "Too many hotspots: {} (max 32767)", count)
            }
            ConversionError::TooManyPictures { count } => {
                write!(f, "Too many pictures: {} (max 32767)", count)
            }
            ConversionError::TooManyPoints { hotspot_id, count } => {
                write!(
                    f,
                    "Too many points in hotspot {}: {} (max 32767)",
                    hotspot_id, count
                )
            }
            ConversionError::TooManyStates { hotspot_id, count } => {
                write!(
                    f,
                    "Too many states in hotspot {}: {} (max 32767)",
                    hotspot_id, count
                )
            }
            ConversionError::StringTooLong { field, length } => {
                write!(
                    f,
                    "String too long for field '{}': {} bytes (max 255)",
                    field, length
                )
            }
            ConversionError::ScriptSerializationError { message } => {
                write!(f, "Script serialization error: {}", message)
            }
        }
    }
}

impl std::error::Error for ConversionError {}

/// Helper for building the varBuf with proper alignment and offset tracking.
struct VarBufBuilder {
    buf: BytesMut,
}

impl VarBufBuilder {
    /// Create a new empty VarBufBuilder.
    fn new() -> Self {
        Self {
            buf: BytesMut::new(),
        }
    }

    /// Get the current offset.
    fn offset(&self) -> usize {
        self.buf.len()
    }

    /// Write a PString (length byte + data) and return the offset.
    fn write_pstring(&mut self, s: &str) -> Result<i16, ConversionError> {
        let bytes = s.as_bytes();
        if bytes.len() > 255 {
            return Err(ConversionError::StringTooLong {
                field: s.to_string(),
                length: bytes.len(),
            });
        }

        let offset = self.offset();
        if offset > i16::MAX as usize {
            return Err(ConversionError::VarBufTooLarge { size: offset });
        }

        self.buf.put_u8(bytes.len() as u8);
        self.buf.put_slice(bytes);

        Ok(offset as i16)
    }

    /// Write an optional PString, returning -1 if None.
    fn write_optional_pstring(&mut self, s: Option<&str>) -> Result<i16, ConversionError> {
        match s {
            Some(s) => self.write_pstring(s),
            None => Ok(-1),
        }
    }

    /// Align the buffer to a 4-byte boundary by padding with zeros.
    fn align_to_4(&mut self) {
        let offset = self.offset();
        let padding = (4 - (offset % 4)) % 4;
        for _ in 0..padding {
            self.buf.put_u8(0);
        }
    }

    /// Write a Point (4 bytes: v, h).
    fn write_point(&mut self, point: &Point) {
        self.buf.put_i16(point.v);
        self.buf.put_i16(point.h);
    }

    /// Write an array of Points and return the offset.
    fn write_points(&mut self, points: &[Point]) -> Result<i16, ConversionError> {
        self.align_to_4();

        let offset = self.offset();
        if offset > i16::MAX as usize {
            return Err(ConversionError::VarBufTooLarge { size: offset });
        }

        for point in points {
            self.write_point(point);
        }

        Ok(offset as i16)
    }

    /// Write a StateRec (6 bytes: pic_id, x_offset, y_offset).
    fn write_state(&mut self, pic_id: i16, x_offset: i16, y_offset: i16) {
        self.buf.put_i16(pic_id);
        self.buf.put_i16(x_offset);
        self.buf.put_i16(y_offset);
    }

    /// Write an array of StateRecs and return the offset.
    fn write_states(
        &mut self,
        states: &[crate::iptscrae::StateDecl],
    ) -> Result<i16, ConversionError> {
        self.align_to_4();

        let offset = self.offset();
        if offset > i16::MAX as usize {
            return Err(ConversionError::VarBufTooLarge { size: offset });
        }

        for state in states {
            self.write_state(state.pic_id, state.x_offset, state.y_offset);
        }

        Ok(offset as i16)
    }

    /// Write a Hotspot structure (48 bytes).
    fn write_hotspot(&mut self, hotspot: &Hotspot) {
        self.buf.put_i32(hotspot.script_event_mask.into());
        self.buf.put_i32(hotspot.flags);
        self.buf.put_i32(hotspot.secure_info);
        self.buf.put_i32(hotspot.ref_con);
        self.write_point(&hotspot.loc);
        self.buf.put_i16(hotspot.id);
        self.buf.put_i16(hotspot.dest);
        self.buf.put_i16(hotspot.nbr_pts);
        self.buf.put_i16(hotspot.pts_ofst);
        self.buf.put_i16(hotspot.hotspot_type.as_i16());
        self.buf.put_i16(hotspot.group_id);
        self.buf.put_i16(hotspot.nbr_scripts);
        self.buf.put_i16(hotspot.script_rec_ofst);
        self.buf.put_i16(hotspot.state.as_i16());
        self.buf.put_i16(hotspot.nbr_states);
        self.buf.put_i16(hotspot.state_rec_ofst);
        self.buf.put_i16(hotspot.name_ofst);
        self.buf.put_i16(hotspot.script_text_ofst);
        self.buf.put_i16(0); // padding
    }

    /// Write an array of Hotspots and return the offset.
    fn write_hotspots(&mut self, hotspots: &[Hotspot]) -> Result<i16, ConversionError> {
        self.align_to_4();

        let offset = self.offset();
        if offset > i16::MAX as usize {
            return Err(ConversionError::VarBufTooLarge { size: offset });
        }

        for hotspot in hotspots {
            self.write_hotspot(hotspot);
        }

        Ok(offset as i16)
    }

    /// Write a PictureRec structure (12 bytes).
    fn write_picture_rec(&mut self, pic: &PictureRec) {
        self.buf.put_i32(pic.ref_con);
        self.buf.put_i16(pic.pic_id);
        self.buf.put_i16(pic.pic_name_ofst);
        self.buf.put_i16(pic.trans_color);
        self.buf.put_i16(0); // padding
    }

    /// Write an array of PictureRecs and return the offset.
    fn write_picture_recs(&mut self, pictures: &[PictureRec]) -> Result<i16, ConversionError> {
        self.align_to_4();

        let offset = self.offset();
        if offset > i16::MAX as usize {
            return Err(ConversionError::VarBufTooLarge { size: offset });
        }

        for pic in pictures {
            self.write_picture_rec(pic);
        }

        Ok(offset as i16)
    }

    /// Finish building and return the final Bytes buffer.
    fn finish(self) -> Bytes {
        self.buf.freeze()
    }
}

/// Convert room script flags to protocol RoomFlags.
fn convert_flags(flags: &crate::iptscrae::RoomFlags) -> crate::messages::flags::RoomFlags {
    use crate::messages::flags::RoomFlags;

    let mut result = RoomFlags::empty();

    if flags.private {
        result |= RoomFlags::PRIVATE;
    }
    if flags.no_painting {
        result |= RoomFlags::NO_PAINTING;
    }
    if flags.no_cyborgs {
        result |= RoomFlags::CYBORG_FREE_ZONE;
    }
    if flags.hidden {
        result |= RoomFlags::HIDDEN;
    }
    if flags.no_guests {
        result |= RoomFlags::NO_GUESTS;
    }

    result
}

/// Extract event mask from a script by collecting all event types.
fn extract_event_mask(script: &Script) -> EventMask {
    let mut mask = EventMask::empty();

    for handler in &script.handlers {
        mask |= handler.event.to_mask();
    }

    mask
}

/// Serialize a script back to Iptscrae source text.
///
/// TODO: This is a placeholder. We need to implement proper script serialization.
#[allow(dead_code)]
fn serialize_script(_script: &Script) -> Result<String, ConversionError> {
    // For now, return a placeholder
    // In the future, implement Script::to_string() or a proper serializer
    Err(ConversionError::ScriptSerializationError {
        message: "Script serialization not yet implemented".to_string(),
    })
}

/// Convert a RoomDecl to a RoomRec template.
pub fn convert_room(room: &RoomDecl) -> Result<RoomRec, ConversionError> {
    let mut var_buf = VarBufBuilder::new();

    // Convert flags
    let room_flags = convert_flags(&room.flags);

    // Write room strings
    let room_name_ofst = var_buf.write_optional_pstring(room.name.as_deref())?;
    let pict_name_ofst = var_buf.write_optional_pstring(room.pict.as_deref())?;
    let artist_name_ofst = var_buf.write_optional_pstring(room.artist.as_deref())?;
    let password_ofst = var_buf.write_optional_pstring(room.password.as_deref())?;

    // Prepare pictures
    let nbr_pictures = room.pictures.len();
    if nbr_pictures > i16::MAX as usize {
        return Err(ConversionError::TooManyPictures {
            count: nbr_pictures,
        });
    }

    let mut picture_recs = Vec::new();
    for pic_decl in &room.pictures {
        let pic_name_ofst = var_buf.write_pstring(&pic_decl.name)?;
        picture_recs.push(PictureRec {
            ref_con: 0,
            pic_id: pic_decl.id,
            pic_name_ofst,
            trans_color: pic_decl.trans_color.unwrap_or(-1),
        });
    }

    let picture_ofst = if picture_recs.is_empty() {
        0
    } else {
        var_buf.write_picture_recs(&picture_recs)?
    };

    // Prepare hotspots (doors + spots)
    let nbr_hotspots = room.doors.len() + room.spots.len();
    if nbr_hotspots > i16::MAX as usize {
        return Err(ConversionError::TooManyHotspots {
            count: nbr_hotspots,
        });
    }

    let mut hotspots = Vec::new();

    // Convert doors
    for door in &room.doors {
        let hotspot = convert_door(door, &mut var_buf)?;
        hotspots.push(hotspot);
    }

    // Convert spots
    for spot in &room.spots {
        let hotspot = convert_spot(spot, &mut var_buf)?;
        hotspots.push(hotspot);
    }

    let hotspot_ofst = if hotspots.is_empty() {
        0
    } else {
        var_buf.write_hotspots(&hotspots)?
    };

    // Finish varBuf
    let var_buf_bytes = var_buf.finish();
    let len_vars = var_buf_bytes.len();
    if len_vars > i16::MAX as usize {
        return Err(ConversionError::VarBufTooLarge { size: len_vars });
    }

    // Create RoomRec
    Ok(RoomRec {
        room_flags,
        faces_id: 0, // Default
        room_id: room.id,
        room_name_ofst,
        pict_name_ofst,
        artist_name_ofst,
        password_ofst,
        nbr_hotspots: nbr_hotspots as i16,
        hotspot_ofst,
        nbr_pictures: nbr_pictures as i16,
        picture_ofst,
        nbr_draw_cmds: 0, // Runtime field
        first_draw_cmd: 0,
        nbr_people: 0, // Runtime field
        nbr_lprops: 0, // Runtime field
        first_lprop: 0,
        len_vars: len_vars as i16,
        var_buf: var_buf_bytes,
    })
}

/// Convert a DoorDecl to a Hotspot.
fn convert_door(
    door: &crate::iptscrae::DoorDecl,
    var_buf: &mut VarBufBuilder,
) -> Result<Hotspot, ConversionError> {
    // Check limits
    if door.outline.len() > i16::MAX as usize {
        return Err(ConversionError::TooManyPoints {
            hotspot_id: door.id,
            count: door.outline.len(),
        });
    }
    if door.picts.len() > i16::MAX as usize {
        return Err(ConversionError::TooManyStates {
            hotspot_id: door.id,
            count: door.picts.len(),
        });
    }

    // Write name
    let name_ofst = var_buf.write_optional_pstring(door.name.as_deref())?;

    // Write outline points
    let pts_ofst = if door.outline.is_empty() {
        0
    } else {
        var_buf.write_points(&door.outline)?
    };

    // Write states
    let state_rec_ofst = if door.picts.is_empty() {
        0
    } else {
        var_buf.write_states(&door.picts)?
    };

    // Handle script
    let (script_event_mask, nbr_scripts, script_rec_ofst, script_text_ofst) =
        if let Some(ref script) = door.script {
            let event_mask = extract_event_mask(script);
            // TODO: Implement script serialization and script records
            // For now, just set the event mask and zero out the rest
            (event_mask, 0, 0, 0)
        } else {
            (EventMask::empty(), 0, 0, 0)
        };

    // Location: use first point or origin
    let loc = door.outline.first().copied().unwrap_or(Point::origin());

    Ok(Hotspot {
        script_event_mask,
        flags: 0,
        secure_info: 0,
        ref_con: 0,
        loc,
        id: door.id,
        dest: door.dest,
        nbr_pts: door.outline.len() as i16,
        pts_ofst,
        hotspot_type: HotspotType::Door,
        group_id: 0,
        nbr_scripts,
        script_rec_ofst,
        state: HotspotState::Unlocked,
        nbr_states: door.picts.len() as i16,
        state_rec_ofst,
        name_ofst,
        script_text_ofst,
    })
}

/// Convert a SpotDecl to a Hotspot.
fn convert_spot(
    spot: &crate::iptscrae::SpotDecl,
    var_buf: &mut VarBufBuilder,
) -> Result<Hotspot, ConversionError> {
    // Check limits
    if spot.outline.len() > i16::MAX as usize {
        return Err(ConversionError::TooManyPoints {
            hotspot_id: spot.id,
            count: spot.outline.len(),
        });
    }
    if spot.picts.len() > i16::MAX as usize {
        return Err(ConversionError::TooManyStates {
            hotspot_id: spot.id,
            count: spot.picts.len(),
        });
    }

    // Write name
    let name_ofst = var_buf.write_optional_pstring(spot.name.as_deref())?;

    // Write outline points
    let pts_ofst = if spot.outline.is_empty() {
        0
    } else {
        var_buf.write_points(&spot.outline)?
    };

    // Write states
    let state_rec_ofst = if spot.picts.is_empty() {
        0
    } else {
        var_buf.write_states(&spot.picts)?
    };

    // Handle script
    let (script_event_mask, nbr_scripts, script_rec_ofst, script_text_ofst) =
        if let Some(ref script) = spot.script {
            let event_mask = extract_event_mask(script);
            // TODO: Implement script serialization and script records
            // For now, just set the event mask and zero out the rest
            (event_mask, 0, 0, 0)
        } else {
            (EventMask::empty(), 0, 0, 0)
        };

    // Location: use first point or origin
    let loc = spot.outline.first().copied().unwrap_or(Point::origin());

    Ok(Hotspot {
        script_event_mask,
        flags: 0,
        secure_info: 0,
        ref_con: 0,
        loc,
        id: spot.id,
        dest: 0, // Spots don't have destinations
        nbr_pts: spot.outline.len() as i16,
        pts_ofst,
        hotspot_type: HotspotType::Normal,
        group_id: 0,
        nbr_scripts,
        script_rec_ofst,
        state: HotspotState::Unlocked,
        nbr_states: spot.picts.len() as i16,
        state_rec_ofst,
        name_ofst,
        script_text_ofst,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iptscrae::room_script::RoomFlags as AstRoomFlags;

    #[test]
    fn test_convert_flags() {
        let flags = AstRoomFlags {
            private: true,
            no_painting: true,
            no_cyborgs: false,
            hidden: false,
            no_guests: false,
        };

        let result = convert_flags(&flags);

        use crate::messages::flags::RoomFlags;
        assert!(result.contains(RoomFlags::PRIVATE));
        assert!(result.contains(RoomFlags::NO_PAINTING));
        assert!(!result.contains(RoomFlags::CYBORG_FREE_ZONE));
        assert!(!result.contains(RoomFlags::HIDDEN));
        assert!(!result.contains(RoomFlags::NO_GUESTS));
    }

    #[test]
    fn test_var_buf_builder_pstring() {
        let mut builder = VarBufBuilder::new();

        let offset1 = builder.write_pstring("Hello").unwrap();
        assert_eq!(offset1, 0);
        assert_eq!(builder.offset(), 6); // 1 byte length + 5 bytes data

        let offset2 = builder.write_pstring("World").unwrap();
        assert_eq!(offset2, 6);
        assert_eq!(builder.offset(), 12);

        let bytes = builder.finish();
        assert_eq!(bytes.len(), 12);
        assert_eq!(bytes[0], 5); // "Hello" length
        assert_eq!(&bytes[1..6], b"Hello");
        assert_eq!(bytes[6], 5); // "World" length
        assert_eq!(&bytes[7..12], b"World");
    }

    #[test]
    fn test_var_buf_builder_optional_pstring() {
        let mut builder = VarBufBuilder::new();

        let offset1 = builder.write_optional_pstring(Some("Test")).unwrap();
        assert_eq!(offset1, 0);

        let offset2 = builder.write_optional_pstring(None).unwrap();
        assert_eq!(offset2, -1);

        assert_eq!(builder.offset(), 5); // Only "Test" was written
    }

    #[test]
    fn test_var_buf_builder_alignment() {
        let mut builder = VarBufBuilder::new();

        builder.write_pstring("Hi").unwrap(); // 3 bytes: length + 2 chars
        assert_eq!(builder.offset(), 3);

        builder.align_to_4();
        assert_eq!(builder.offset(), 4); // Padded to 4-byte boundary

        builder.write_pstring("Test").unwrap(); // 5 bytes
        assert_eq!(builder.offset(), 9);

        builder.align_to_4();
        assert_eq!(builder.offset(), 12); // Padded to next 4-byte boundary
    }

    #[test]
    fn test_var_buf_builder_points() {
        let mut builder = VarBufBuilder::new();

        let points = vec![
            Point { h: 10, v: 20 },
            Point { h: 30, v: 40 },
            Point { h: 50, v: 60 },
        ];

        let offset = builder.write_points(&points).unwrap();
        assert_eq!(offset, 0); // Aligned to start

        let bytes = builder.finish();
        assert_eq!(bytes.len(), 12); // 3 points × 4 bytes
    }

    #[test]
    fn test_var_buf_builder_states() {
        let mut builder = VarBufBuilder::new();

        use crate::iptscrae::StateDecl;
        let states = vec![
            StateDecl {
                pic_id: 100,
                x_offset: 10,
                y_offset: -5,
            },
            StateDecl {
                pic_id: 101,
                x_offset: 0,
                y_offset: 0,
            },
        ];

        let offset = builder.write_states(&states).unwrap();
        assert_eq!(offset, 0);

        let bytes = builder.finish();
        assert_eq!(bytes.len(), 12); // 2 states × 6 bytes
    }

    #[test]
    fn test_string_too_long() {
        let mut builder = VarBufBuilder::new();
        let long_string = "a".repeat(256);

        let result = builder.write_pstring(&long_string);
        assert!(matches!(result, Err(ConversionError::StringTooLong { .. })));
    }

    #[test]
    fn test_convert_simple_room() {
        use crate::iptscrae::RoomDecl;

        let room = RoomDecl {
            id: 100,
            name: Some("Test Room".to_string()),
            pict: Some("background.gif".to_string()),
            artist: Some("Artist Name".to_string()),
            password: None,
            flags: AstRoomFlags {
                private: true,
                no_painting: false,
                no_cyborgs: false,
                hidden: false,
                no_guests: false,
            },
            pictures: vec![],
            doors: vec![],
            spots: vec![],
        };

        let result = convert_room(&room).unwrap();

        assert_eq!(result.room_id, 100);
        assert_eq!(result.nbr_hotspots, 0);
        assert_eq!(result.nbr_pictures, 0);
        assert_eq!(result.nbr_people, 0); // Runtime field
        assert_eq!(result.nbr_lprops, 0); // Runtime field
        assert_eq!(result.nbr_draw_cmds, 0); // Runtime field

        use crate::messages::flags::RoomFlags;
        assert!(result.room_flags.contains(RoomFlags::PRIVATE));

        // Verify strings in varBuf
        assert_eq!(result.room_name().unwrap(), "Test Room");
        assert_eq!(result.pict_name().unwrap(), "background.gif");
        assert_eq!(result.artist_name().unwrap(), "Artist Name");
    }

    #[test]
    fn test_convert_room_with_door() {
        use crate::iptscrae::{DoorDecl, RoomDecl};

        let door = DoorDecl {
            id: 1,
            dest: 200,
            name: Some("Exit".to_string()),
            outline: vec![
                Point { h: 10, v: 10 },
                Point { h: 50, v: 10 },
                Point { h: 50, v: 100 },
                Point { h: 10, v: 100 },
            ],
            picts: vec![],
            script: None,
        };

        let room = RoomDecl {
            id: 100,
            name: Some("Room with Door".to_string()),
            pict: None,
            artist: None,
            password: None,
            flags: AstRoomFlags::default(),
            pictures: vec![],
            doors: vec![door],
            spots: vec![],
        };

        let result = convert_room(&room).unwrap();

        assert_eq!(result.room_id, 100);
        assert_eq!(result.nbr_hotspots, 1);

        // The hotspot should be a door
        // We can't easily verify hotspot details without parsing varBuf
        // but we can check that it was created
        assert!(result.hotspot_ofst > 0);
    }

    #[test]
    fn test_convert_room_with_pictures() {
        use crate::iptscrae::{PictureDecl, RoomDecl};

        let pictures = vec![
            PictureDecl {
                id: 1,
                name: "overlay1.gif".to_string(),
                trans_color: Some(255),
            },
            PictureDecl {
                id: 2,
                name: "overlay2.gif".to_string(),
                trans_color: None,
            },
        ];

        let room = RoomDecl {
            id: 100,
            name: Some("Room with Pictures".to_string()),
            pict: None,
            artist: None,
            password: None,
            flags: AstRoomFlags::default(),
            pictures,
            doors: vec![],
            spots: vec![],
        };

        let result = convert_room(&room).unwrap();

        assert_eq!(result.room_id, 100);
        assert_eq!(result.nbr_pictures, 2);
        assert!(result.picture_ofst > 0);
    }

    #[test]
    fn test_convert_room_with_spot() {
        use crate::iptscrae::{RoomDecl, SpotDecl};

        let spot = SpotDecl {
            id: 2,
            name: Some("Button".to_string()),
            outline: vec![
                Point { h: 100, v: 100 },
                Point { h: 200, v: 100 },
                Point { h: 200, v: 200 },
                Point { h: 100, v: 200 },
            ],
            picts: vec![],
            script: None,
        };

        let room = RoomDecl {
            id: 100,
            name: Some("Room with Spot".to_string()),
            pict: None,
            artist: None,
            password: None,
            flags: AstRoomFlags::default(),
            pictures: vec![],
            doors: vec![],
            spots: vec![spot],
        };

        let result = convert_room(&room).unwrap();

        assert_eq!(result.room_id, 100);
        assert_eq!(result.nbr_hotspots, 1);
        assert!(result.hotspot_ofst > 0);
    }

    #[test]
    fn test_convert_room_all_features() {
        use crate::iptscrae::{DoorDecl, PictureDecl, RoomDecl, SpotDecl, StateDecl};

        let room = RoomDecl {
            id: 42,
            name: Some("Complete Room".to_string()),
            pict: Some("bg.gif".to_string()),
            artist: Some("Test Artist".to_string()),
            password: Some("secret".to_string()),
            flags: AstRoomFlags {
                private: true,
                no_painting: true,
                no_cyborgs: true,
                hidden: true,
                no_guests: true,
            },
            pictures: vec![PictureDecl {
                id: 10,
                name: "layer.gif".to_string(),
                trans_color: Some(255),
            }],
            doors: vec![DoorDecl {
                id: 1,
                dest: 100,
                name: Some("Door".to_string()),
                outline: vec![Point { h: 0, v: 0 }, Point { h: 10, v: 10 }],
                picts: vec![StateDecl {
                    pic_id: 50,
                    x_offset: 5,
                    y_offset: -3,
                }],
                script: None,
            }],
            spots: vec![SpotDecl {
                id: 2,
                name: Some("Spot".to_string()),
                outline: vec![Point { h: 20, v: 20 }, Point { h: 30, v: 30 }],
                picts: vec![],
                script: None,
            }],
        };

        let result = convert_room(&room).unwrap();

        assert_eq!(result.room_id, 42);
        assert_eq!(result.nbr_hotspots, 2); // 1 door + 1 spot
        assert_eq!(result.nbr_pictures, 1);

        use crate::messages::flags::RoomFlags;
        assert!(result.room_flags.contains(RoomFlags::PRIVATE));
        assert!(result.room_flags.contains(RoomFlags::NO_PAINTING));
        assert!(result.room_flags.contains(RoomFlags::CYBORG_FREE_ZONE));
        assert!(result.room_flags.contains(RoomFlags::HIDDEN));
        assert!(result.room_flags.contains(RoomFlags::NO_GUESTS));

        assert_eq!(result.room_name().unwrap(), "Complete Room");
        assert_eq!(result.pict_name().unwrap(), "bg.gif");
        assert_eq!(result.artist_name().unwrap(), "Test Artist");
        assert_eq!(result.password().unwrap(), "secret");
    }

    #[test]
    fn test_extract_event_mask_empty() {
        let script = Script { handlers: vec![] };
        let mask = extract_event_mask(&script);
        assert_eq!(mask, EventMask::empty());
    }

    #[test]
    fn test_extract_event_mask_multiple() {
        use crate::iptscrae::{Block, EventHandler, EventType, SourcePos};

        let script = Script {
            handlers: vec![
                EventHandler {
                    event: EventType::Select,
                    body: Block { statements: vec![] },
                    pos: SourcePos { line: 1, column: 1 },
                },
                EventHandler {
                    event: EventType::Enter,
                    body: Block { statements: vec![] },
                    pos: SourcePos { line: 2, column: 1 },
                },
                EventHandler {
                    event: EventType::Leave,
                    body: Block { statements: vec![] },
                    pos: SourcePos { line: 3, column: 1 },
                },
            ],
        };

        let mask = extract_event_mask(&script);
        assert!(mask.contains(EventMask::SELECT));
        assert!(mask.contains(EventMask::ENTER));
        assert!(mask.contains(EventMask::LEAVE));
        assert!(!mask.contains(EventMask::LOCK));
    }
}
