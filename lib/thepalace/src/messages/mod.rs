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
pub mod message;
pub mod message_id;

pub use flags::*;
pub use message::Message;
pub use message_id::MessageId;

// TODO: Implement specific message payload types
// - Authentication messages (TIYID, LOGON, AUTHENTICATE, etc.)
// - Room messages (ROOMGOTO, ROOMDESC, etc.)
// - User messages (USERNEW, USEREXIT, USERMOVE, etc.)
// - Chat messages (TALK, WHISPER, XTALK, etc.)
// - Asset messages (ASSETQUERY, ASSETSEND, ASSETREGI)
// - And 50+ more message types
