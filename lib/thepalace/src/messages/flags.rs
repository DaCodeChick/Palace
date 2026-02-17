//! Bitflags for Palace Protocol entities.
//!
//! This module defines various flag sets used throughout the Palace Protocol
//! for users, rooms, props, servers, and other entities.

use bitflags::bitflags;

bitflags! {
    /// User flags describing user state and permissions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct UserFlags: u16 {
        /// Wizard (limited admin)
        const SUPERUSER = 0x0001;
        /// God (full admin)
        const GOD = 0x0002;
        /// Server should drop user at first opportunity
        const KILL = 0x0004;
        /// User is a guest (no registration code)
        const GUEST = 0x0008;
        /// Redundant with KILL, shouldn't be used
        const BANISHED = 0x0010;
        /// Historical artifact, shouldn't be used
        const PENALIZED = 0x0020;
        /// Communication error, drop at first opportunity
        const COMM_ERROR = 0x0040;
        /// Not allowed to speak
        const GAG = 0x0080;
        /// Stuck in corner and not allowed to move
        const PIN = 0x0100;
        /// Doesn't appear on user list
        const HIDE = 0x0200;
        /// Rejects ESP (Enhanced Sensory Perception) messages
        const REJECT_ESP = 0x0400;
        /// Rejects private messages
        const REJECT_PRIVATE = 0x0800;
        /// Not allowed to change props
        const PROP_GAG = 0x1000;
    }
}

bitflags! {
    /// Room flags describing room attributes and restrictions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RoomFlags: u16 {
        /// Only author can enter
        const AUTHOR_LOCKED = 0x0001;
        /// Private room
        const PRIVATE = 0x0002;
        /// Drawing/painting disabled
        const NO_PAINTING = 0x0004;
        /// Room is closed
        const CLOSED = 0x0008;
        /// Cyborg scripts disabled in this room
        const CYBORG_FREE_ZONE = 0x0010;
        /// Hidden from room list
        const HIDDEN = 0x0020;
        /// Guest users not allowed
        const NO_GUESTS = 0x0040;
        /// Only wizards allowed
        const WIZARDS_ONLY = 0x0080;
        /// Drop zone for props
        const DROP_ZONE = 0x0100;
        /// Loose props disabled
        const NO_LOOSE_PROPS = 0x0200;
    }
}

bitflags! {
    /// Prop flags describing prop format and behavior.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PropFlags: u16 {
        /// 8-bit indexed color format (default, 0x0000)
        const FORMAT_8BIT = 0x0000;
        /// Prop is a head/face prop
        const HEAD = 0x0002;
        /// Ghost mode (transparent/overlay)
        const GHOST = 0x0004;
        /// Rare prop
        const RARE = 0x0008;
        /// Animated prop
        const ANIMATE = 0x0010;
        /// Bouncing prop
        const BOUNCE = 0x0020;
        /// 20-bit color format
        const FORMAT_20BIT = 0x0040;
        /// 32-bit RGBA color format
        const FORMAT_32BIT = 0x0100;
        /// Signed 20-bit color format
        const FORMAT_S20BIT = 0x0200;
    }
}

impl PropFlags {
    /// Get the color format bits from the flags.
    pub fn format(&self) -> PropFormat {
        if self.contains(Self::FORMAT_32BIT) {
            PropFormat::Rgb32
        } else if self.contains(Self::FORMAT_S20BIT) {
            PropFormat::S20Bit
        } else if self.contains(Self::FORMAT_20BIT) {
            PropFormat::Rgb20
        } else {
            PropFormat::Indexed8
        }
    }
}

/// Prop color format enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PropFormat {
    /// 8-bit indexed color (palette-based)
    Indexed8 = 0,
    /// 20-bit RGB color
    Rgb20 = 1,
    /// 32-bit RGBA color
    Rgb32 = 2,
    /// Signed 20-bit color
    S20Bit = 3,
}

bitflags! {
    /// Server flags describing server configuration and capabilities.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ServerFlags: u32 {
        /// DirectPlay enabled
        const DIRECT_PLAY = 0x0001;
        /// Closed server (registration required)
        const CLOSED_SERVER = 0x0002;
        /// Guests are treated as members
        const GUESTS_ARE_MEMBERS = 0x0004;
        /// Unused flag bit
        const UNUSED_1 = 0x0008;
        /// InstantPalace server
        const INSTANT_PALACE = 0x0010;
        /// PalacePresents branding
        const PALACE_PRESENTS = 0x0020;
        /// Allow cyborg (client-side bot) scripts globally
        const ALLOW_CYBORGS = 0x0200;
    }
}

bitflags! {
    /// Iptscrae script event flags indicating which events trigger a script.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ScriptEventFlags: u32 {
        /// Hotspot clicked/selected
        const SELECT = 0x00000001;
        /// Door locked
        const LOCK = 0x00000002;
        /// Door unlocked
        const UNLOCK = 0x00000004;
        /// Hotspot hidden
        const HIDE = 0x00000008;
        /// Hotspot shown
        const SHOW = 0x00000010;
        /// Room startup
        const STARTUP = 0x00000020;
        /// Timer alarm triggered
        const ALARM = 0x00000040;
        /// Custom event
        const CUSTOM = 0x00000080;
        /// Chat message received
        const INCHAT = 0x00000100;
        /// Prop changed
        const PROPCHANGE = 0x00000200;
        /// User entered room
        const ENTER = 0x00000400;
        /// User left room
        const LEAVE = 0x00000800;
        /// Chat message sent
        const OUTCHAT = 0x00001000;
        /// User signed on
        const SIGNON = 0x00002000;
        /// User signed off
        const SIGNOFF = 0x00004000;
        /// Macro 0 triggered
        const MACRO0 = 0x00008000;
        /// Macro 1 triggered
        const MACRO1 = 0x00010000;
        /// Macro 2 triggered
        const MACRO2 = 0x00020000;
        /// Macro 3 triggered
        const MACRO3 = 0x00040000;
        /// Macro 4 triggered
        const MACRO4 = 0x00080000;
        /// Macro 5 triggered
        const MACRO5 = 0x00100000;
        /// Macro 6 triggered
        const MACRO6 = 0x00200000;
        /// Macro 7 triggered
        const MACRO7 = 0x00400000;
        /// Macro 8 triggered
        const MACRO8 = 0x00800000;
        /// Macro 9 triggered
        const MACRO9 = 0x01000000;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_flags() {
        let mut flags = UserFlags::GUEST;
        assert!(flags.contains(UserFlags::GUEST));
        assert!(!flags.contains(UserFlags::SUPERUSER));

        flags |= UserFlags::GAG;
        assert!(flags.contains(UserFlags::GUEST));
        assert!(flags.contains(UserFlags::GAG));
    }

    #[test]
    fn test_room_flags() {
        let flags = RoomFlags::WIZARDS_ONLY | RoomFlags::NO_GUESTS;
        assert!(flags.contains(RoomFlags::WIZARDS_ONLY));
        assert!(flags.contains(RoomFlags::NO_GUESTS));
        assert!(!flags.contains(RoomFlags::PRIVATE));
    }

    #[test]
    fn test_prop_format() {
        let flags_8bit = PropFlags::FORMAT_8BIT | PropFlags::HEAD;
        assert_eq!(flags_8bit.format(), PropFormat::Indexed8);

        let flags_32bit = PropFlags::FORMAT_32BIT | PropFlags::ANIMATE;
        assert_eq!(flags_32bit.format(), PropFormat::Rgb32);
    }

    #[test]
    fn test_server_flags() {
        let flags = ServerFlags::CLOSED_SERVER | ServerFlags::ALLOW_CYBORGS;
        assert!(flags.contains(ServerFlags::CLOSED_SERVER));
        assert!(flags.contains(ServerFlags::ALLOW_CYBORGS));
        assert!(!flags.contains(ServerFlags::INSTANT_PALACE));
    }

    #[test]
    fn test_script_event_flags() {
        let events = ScriptEventFlags::SELECT | ScriptEventFlags::ENTER | ScriptEventFlags::LEAVE;
        assert!(events.contains(ScriptEventFlags::SELECT));
        assert!(events.contains(ScriptEventFlags::ENTER));
        assert!(!events.contains(ScriptEventFlags::ALARM));
    }
}
