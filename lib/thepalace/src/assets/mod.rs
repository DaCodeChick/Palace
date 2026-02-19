//! Asset management for Palace props, backgrounds, and other media.
//!
//! Assets in the Palace Protocol are identified by CRC32 checksums and stored
//! on the filesystem with the checksum as the filename.
//!
//! ## Storage Layout
//!
//! - Props: `assets/props/{CRC32_HEX}.prop`
//! - Backgrounds: `assets/backgrounds/{CRC32_HEX}.{png,jpg}`
//! - Other assets as needed
//!
//! ## Prop Formats
//!
//! The Palace Protocol supports multiple prop formats. See the [`crate::prop`] module
//! for full implementation details:
//!
//! - **8-bit**: Indexed color with palette (run-length encoded)
//! - **20-bit**: RGB color with 1-bit alpha (6+6+6+2 bits/pixel, zlib compressed)
//! - **32-bit**: RGBA color (8+8+8+8 bits/pixel, zlib compressed)
//! - **S20-bit**: Special 20-bit format (5+5+5+5 bits/pixel, zlib compressed)
//!
//! All props are typically 44x44 pixels and include a 12-byte header with metadata.

// TODO: Implement asset management
// - Asset storage and retrieval
// - Asset upload/download protocol
// - CRC32-based asset identification
