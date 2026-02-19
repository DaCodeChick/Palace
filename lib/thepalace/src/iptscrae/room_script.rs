//! Room script AST for Palace server script files.
//!
//! This module handles the meta-syntax for defining rooms, doors, and spots
//! in Palace server script files (e.g., Mansion.ipt). This is separate from
//! the regular Iptscrae script execution language.
//!
//! ## Example
//!
//! ```text
//! ROOM
//!   ID 100
//!   NAME "Entrance Hall"
//!   PICT "entrance.gif"
//!   ARTIST "Jane Doe"
//!   PRIVATE
//!   
//!   DOOR
//!     ID 1
//!     DEST 200
//!     OUTLINE 10,10 50,10 50,200 10,200
//!   ENDDOOR
//!   
//!   SPOT
//!     ID 2
//!     NAME "Button"
//!     OUTLINE 100,100 200,100 200,200 100,200
//!     SCRIPT
//!       ON SELECT { "You clicked!" SAY }
//!     ENDSCRIPT
//!   ENDSPOT
//! ENDROOM
//! ```

use crate::iptscrae::Script;
use crate::Point;

/// Complete room declaration in a server script file.
#[derive(Debug, Clone, PartialEq)]
pub struct RoomDecl {
    /// Room ID number (required, must be unique)
    pub id: i16,
    /// Room name (optional)
    pub name: Option<String>,
    /// Background picture filename (optional)
    pub pict: Option<String>,
    /// Artist name (optional)
    pub artist: Option<String>,
    /// Room password (optional)
    pub password: Option<String>,
    /// Room flags
    pub flags: RoomFlags,
    /// Additional picture layers
    pub pictures: Vec<PictureDecl>,
    /// Door hotspots (doors are special spots that link to other rooms)
    pub doors: Vec<DoorDecl>,
    /// Regular hotspots
    pub spots: Vec<SpotDecl>,
}

/// Room flags that can be set in the room declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RoomFlags {
    /// Room is private (user count hidden in room list)
    pub private: bool,
    /// Painting commands are prohibited
    pub no_painting: bool,
    /// User scripts (cyborgs) are prohibited
    pub no_cyborgs: bool,
    /// Room is hidden from room list
    pub hidden: bool,
    /// Guests are not allowed
    pub no_guests: bool,
}

/// Picture layer declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct PictureDecl {
    /// Picture ID number
    pub id: i16,
    /// Picture filename
    pub name: String,
    /// Transparent color (optional)
    pub trans_color: Option<i16>,
}

/// Door hotspot declaration (links to another room).
#[derive(Debug, Clone, PartialEq)]
pub struct DoorDecl {
    /// Door ID number (required, must be unique within room)
    pub id: i16,
    /// Destination room ID
    pub dest: i16,
    /// Door name (optional)
    pub name: Option<String>,
    /// Polygon outline points
    pub outline: Vec<Point>,
    /// Door states (pictures)
    pub picts: Vec<StateDecl>,
    /// Script attached to this door (optional)
    pub script: Option<Script>,
}

/// Regular hotspot declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct SpotDecl {
    /// Spot ID number (required, must be unique within room)
    pub id: i16,
    /// Spot name (optional)
    pub name: Option<String>,
    /// Polygon outline points
    pub outline: Vec<Point>,
    /// Spot states (pictures)
    pub picts: Vec<StateDecl>,
    /// Script attached to this spot (optional)
    pub script: Option<Script>,
}

/// State declaration (picture with offset).
///
/// Each state is defined by a picture ID and X/Y offsets relative to the
/// hotspot's location. State 0 is the first in the list, state 1 is second, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateDecl {
    /// Picture ID to display in this state
    pub pic_id: i16,
    /// X offset relative to hotspot location (negative = left, positive = right)
    pub x_offset: i16,
    /// Y offset relative to hotspot location (negative = up, positive = down)
    pub y_offset: i16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_decl_creation() {
        let room = RoomDecl {
            id: 100,
            name: Some("Test Room".to_string()),
            pict: Some("test.gif".to_string()),
            artist: Some("Test Artist".to_string()),
            password: None,
            flags: RoomFlags::default(),
            pictures: vec![],
            doors: vec![],
            spots: vec![],
        };

        assert_eq!(room.id, 100);
        assert_eq!(room.name, Some("Test Room".to_string()));
    }

    #[test]
    fn test_room_flags() {
        let flags = RoomFlags {
            private: true,
            no_painting: true,
            no_cyborgs: false,
            hidden: false,
            no_guests: false,
        };

        assert!(flags.private);
        assert!(flags.no_painting);
        assert!(!flags.no_cyborgs);
    }

    #[test]
    fn test_door_decl_creation() {
        let door = DoorDecl {
            id: 1,
            dest: 200,
            name: Some("Exit".to_string()),
            outline: vec![
                Point { h: 10, v: 10 },
                Point { h: 50, v: 10 },
                Point { h: 50, v: 200 },
                Point { h: 10, v: 200 },
            ],
            picts: vec![],
            script: None,
        };

        assert_eq!(door.id, 1);
        assert_eq!(door.dest, 200);
        assert_eq!(door.outline.len(), 4);
    }

    #[test]
    fn test_spot_decl_creation() {
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

        assert_eq!(spot.id, 2);
        assert_eq!(spot.name, Some("Button".to_string()));
        assert_eq!(spot.outline.len(), 4);
    }

    #[test]
    fn test_state_decl() {
        let state = StateDecl {
            pic_id: 100,
            x_offset: 10,
            y_offset: -5,
        };

        assert_eq!(state.pic_id, 100);
        assert_eq!(state.x_offset, 10);
        assert_eq!(state.y_offset, -5);
    }

    #[test]
    fn test_picture_decl() {
        let pic = PictureDecl {
            id: 42,
            name: "overlay.gif".to_string(),
            trans_color: Some(255),
        };

        assert_eq!(pic.id, 42);
        assert_eq!(pic.name, "overlay.gif");
        assert_eq!(pic.trans_color, Some(255));
    }
}
