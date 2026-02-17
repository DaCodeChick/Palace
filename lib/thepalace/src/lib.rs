//! # The Palace Protocol Library
//!
//! This library provides complete implementation of The Palace visual chat protocol,
//! including message types, Iptscrae scripting language, asset handling, and room formats.
//!
//! ## Features
//!
//! - `net` - Network protocol types and message parsing
//! - `prop` - Prop format handling (8-bit, 20-bit, 32-bit)
//! - `iptscrae` - Iptscrae scripting language interpreter
//! - `assets` - Asset management and parsing
//! - `room` - Room format parsing (.ipr files)
//! - `ffi` - C FFI bindings for C++ client
//!
//! ## Example
//!
//! ```rust
//! use thepalace::{Point, AssetSpec};
//!
//! let origin = Point { v: 0, h: 0 };
//! let spec = AssetSpec { id: 1, crc: 0xA95ADE76 };
//! ```

use cfg_if::cfg_if;
use std::fmt;

#[cfg(feature = "net")]
pub mod messages;

#[cfg(feature = "iptscrae")]
pub mod iptscrae;

#[cfg(feature = "assets")]
pub mod assets;

#[cfg(feature = "room")]
pub mod room;

pub mod algo;

cfg_if! {
    if #[cfg(feature = "net")] {
        pub mod buffer;
        pub use buffer::*;
    }
}

#[cfg(feature = "ffi")]
pub mod ffi;

// Re-export commonly used types
pub use algo::{crc32, crypt, pseudo_crc32, PalaceCryptError};

/// A point in 2D space using Mac-style coordinates
///
/// In Palace, coordinates use the Mac convention:
/// - `v` (vertical) increases downward from top of screen
/// - `h` (horizontal) increases rightward from left of screen
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Point {
    /// Vertical coordinate (Y-axis, positive down)
    pub v: i16,
    /// Horizontal coordinate (X-axis, positive right)
    pub h: i16,
}

impl Point {
    /// Create a new point at the given coordinates
    pub const fn new(h: i16, v: i16) -> Self {
        Self { v, h }
    }

    /// Origin point (0, 0)
    pub const fn origin() -> Self {
        Self { v: 0, h: 0 }
    }

    /// Calculate distance to another point
    pub fn distance_to(&self, other: &Point) -> f32 {
        let dh = (other.h - self.h) as f32;
        let dv = (other.v - self.v) as f32;
        (dh * dh + dv * dv).sqrt()
    }

    /// Add two points (vector addition)
    pub fn add(&self, other: &Point) -> Self {
        Self {
            v: self.v.saturating_add(other.v),
            h: self.h.saturating_add(other.h),
        }
    }

    /// Subtract two points (vector subtraction)
    pub fn sub(&self, other: &Point) -> Self {
        Self {
            v: self.v.saturating_sub(other.v),
            h: self.h.saturating_sub(other.h),
        }
    }
}

/// Asset specification - identifies an asset by ID and CRC
///
/// Assets (props, backgrounds, etc.) are identified by a unique ID within
/// their type namespace, and verified using a CRC32 checksum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct AssetSpec {
    /// Asset ID number
    pub id: i32,
    /// CRC32 checksum for verification
    pub crc: u32,
}

impl AssetSpec {
    /// Create a new asset spec
    pub const fn new(id: i32, crc: u32) -> Self {
        Self { id, crc }
    }

    /// Check if CRC is "don't care" (0 means no verification)
    pub fn crc_is_dont_care(&self) -> bool {
        self.crc == 0
    }
}

/// Asset type identifier (4-character ASCII code).
///
/// Asset types are represented as big-endian u32 values where each byte
/// corresponds to an ASCII character, similar to MessageId.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct AssetType(pub u32);

impl AssetType {
    /// Prop asset ('Prop' = 0x50726f70)
    pub const PROP: Self = Self(0x50726f70);

    /// User database asset ('User' = 0x55736572)
    pub const USERBASE: Self = Self(0x55736572);

    /// IP user database asset ('IUsr' = 0x49557372) - historical artifact
    pub const IPUSERBASE: Self = Self(0x49557372);

    /// Convert AssetType to its 4-character ASCII representation
    pub fn as_str(&self) -> String {
        let bytes = self.0.to_be_bytes();
        String::from_utf8_lossy(&bytes).to_string()
    }

    /// Create AssetType from 4-character ASCII string
    pub fn from_str(s: &str) -> Option<Self> {
        if s.len() != 4 {
            return None;
        }
        let bytes = s.as_bytes();
        let value = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        Some(Self(value))
    }

    /// Get the raw u32 value
    pub const fn as_u32(&self) -> u32 {
        self.0
    }
}

impl From<u32> for AssetType {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<AssetType> for u32 {
    fn from(asset: AssetType) -> u32 {
        asset.0
    }
}

impl fmt::Display for AssetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// User ID type
pub type UserID = i32;

/// Room ID type
pub type RoomID = i16;

/// Hotspot ID type
pub type HotspotID = i16;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_creation() {
        let p = Point::new(100, 200);
        assert_eq!(p.h, 100);
        assert_eq!(p.v, 200);
    }

    #[test]
    fn test_point_origin() {
        let origin = Point::origin();
        assert_eq!(origin.h, 0);
        assert_eq!(origin.v, 0);
    }

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0, 0);
        let p2 = Point::new(3, 4);
        assert_eq!(p1.distance_to(&p2), 5.0);
    }

    #[test]
    fn test_point_add() {
        let p1 = Point::new(10, 20);
        let p2 = Point::new(5, 15);
        let result = p1.add(&p2);
        assert_eq!(result.h, 15);
        assert_eq!(result.v, 35);
    }

    #[test]
    fn test_asset_spec() {
        let spec = AssetSpec::new(123, 0xA95ADE76);
        assert_eq!(spec.id, 123);
        assert_eq!(spec.crc, 0xA95ADE76);
        assert!(!spec.crc_is_dont_care());
    }

    #[test]
    fn test_asset_spec_dont_care() {
        let spec = AssetSpec::new(123, 0);
        assert!(spec.crc_is_dont_care());
    }

    #[test]
    fn test_asset_types() {
        // Verify 4-char ASCII codes
        assert_eq!(AssetType::PROP.as_u32(), 0x50726f70);

        // Can convert back to string for debugging
        let bytes = AssetType::PROP.as_u32().to_be_bytes();
        assert_eq!(&bytes, b"Prop");

        // Test string conversion
        assert_eq!(AssetType::PROP.as_str(), "Prop");
        assert_eq!(AssetType::from_str("Prop"), Some(AssetType::PROP));
    }
}
