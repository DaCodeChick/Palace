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
//! The Palace Protocol supports multiple prop formats:
//! - **8-bit**: Indexed color with palette (initial focus)
//! - **20-bit**: RGB color with 1-bit alpha
//! - **32-bit**: RGBA color
//! - **S20-bit**: Special 20-bit format

// TODO: Implement asset management
// - Asset storage and retrieval
// - Prop format parsing (8-bit, 20-bit, 32-bit, S20-bit)
// - CRC32-based asset identification
// - Asset upload/download protocol
