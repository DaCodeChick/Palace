//! Room data structures and management.
//!
//! Rooms are the core spatial unit in the Palace Protocol. Each room contains:
//! - Background image
//! - Hotspots (interactive areas)
//! - Loose props (decorative objects)
//! - Pictures (layered images)
//! - Scripts (Iptscrae event handlers)
//! - Door links to other rooms

/// Hotspot type enumeration.
///
/// Hotspots are interactive areas within a room that can trigger scripts,
/// navigate between rooms, or control access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i16)]
pub enum HotspotType {
    /// Normal hotspot - just a script holder
    Normal = 0,
    /// Door - navigates to another room
    Door = 1,
    /// Shutable door - can be opened/closed
    ShutableDoor = 2,
    /// Lockable door - can be locked/unlocked
    LockableDoor = 3,
    /// Bolt - locks/unlocks other doors
    Bolt = 4,
    /// Navigation area - movement hint
    NavArea = 5,
}

impl HotspotType {
    /// Create HotspotType from i16 value
    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            0 => Some(HotspotType::Normal),
            1 => Some(HotspotType::Door),
            2 => Some(HotspotType::ShutableDoor),
            3 => Some(HotspotType::LockableDoor),
            4 => Some(HotspotType::Bolt),
            5 => Some(HotspotType::NavArea),
            _ => None,
        }
    }

    /// Get the i16 value
    pub const fn as_i16(&self) -> i16 {
        *self as i16
    }

    /// Check if this hotspot type is a door variant
    pub fn is_door(&self) -> bool {
        matches!(
            self,
            HotspotType::Door | HotspotType::ShutableDoor | HotspotType::LockableDoor
        )
    }
}

impl From<HotspotType> for i16 {
    fn from(ht: HotspotType) -> i16 {
        ht as i16
    }
}

/// Hotspot state enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i16)]
pub enum HotspotState {
    /// Unlocked/open
    Unlocked = 0,
    /// Locked/closed
    Locked = 1,
}

impl HotspotState {
    /// Create HotspotState from i16 value
    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            0 => Some(HotspotState::Unlocked),
            1 => Some(HotspotState::Locked),
            _ => None,
        }
    }

    /// Get the i16 value
    pub const fn as_i16(&self) -> i16 {
        *self as i16
    }
}

impl From<HotspotState> for i16 {
    fn from(state: HotspotState) -> i16 {
        state as i16
    }
}

// TODO: Implement room data structures
// - RoomRec structure
// - Hotspot structure
// - Loose props
// - Pictures
// - Room scripts
// - Door links

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotspot_type() {
        assert_eq!(HotspotType::Normal.as_i16(), 0);
        assert_eq!(HotspotType::Door.as_i16(), 1);
        assert_eq!(HotspotType::NavArea.as_i16(), 5);

        assert_eq!(HotspotType::from_i16(0), Some(HotspotType::Normal));
        assert_eq!(HotspotType::from_i16(3), Some(HotspotType::LockableDoor));
        assert_eq!(HotspotType::from_i16(99), None);
    }

    #[test]
    fn test_hotspot_is_door() {
        assert!(!HotspotType::Normal.is_door());
        assert!(HotspotType::Door.is_door());
        assert!(HotspotType::ShutableDoor.is_door());
        assert!(HotspotType::LockableDoor.is_door());
        assert!(!HotspotType::Bolt.is_door());
        assert!(!HotspotType::NavArea.is_door());
    }

    #[test]
    fn test_hotspot_state() {
        assert_eq!(HotspotState::Unlocked.as_i16(), 0);
        assert_eq!(HotspotState::Locked.as_i16(), 1);

        assert_eq!(HotspotState::from_i16(0), Some(HotspotState::Unlocked));
        assert_eq!(HotspotState::from_i16(1), Some(HotspotState::Locked));
        assert_eq!(HotspotState::from_i16(2), None);
    }
}
