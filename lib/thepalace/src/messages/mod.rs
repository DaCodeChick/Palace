//! Palace Protocol network message types.
//!
//! The Palace Protocol uses a generic message structure:
//! - 4 bytes: Event type (big-endian u32, typically ASCII 4-char code)
//! - 4 bytes: Message length (big-endian u32, excluding header)
//! - 4 bytes: Reference number (big-endian i32)
//! - Variable: Payload data
//!
//! This module contains message type identifiers, bitflags, and message structure
//! implementations for all 60+ Palace Protocol message types.

pub mod flags;
pub mod message_id;

pub use flags::*;
pub use message_id::MessageId;

// TODO: Implement message structures
// - ClientMsg / ServerMsg base structures
// - Specific message payload types for each MessageId
// - Message parsing and serialization
