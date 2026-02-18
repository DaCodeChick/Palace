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

pub mod admin;
pub mod asset;
pub mod auth;
pub mod chat;
pub mod flags;
pub mod message;
pub mod message_id;
pub mod room;
pub mod server;
pub mod user;

pub use admin::*;
pub use asset::*;
pub use auth::*;
pub use chat::*;
pub use flags::*;
pub use message::{Message, MessagePayload};
pub use message_id::MessageId;
pub use room::*;
pub use server::*;
pub use user::*;

// TODO: Implement remaining message payload types
// - Door operations (DOORLOCK, DOORUNLOCK)
// - Protocol messages (VERSION, USERSTATUS, NAVERROR, AUTHENTICATE, AUTHRESPONSE)
// - File/display operations (DISPLAYURL, DRAW, FILEQUERY, FILESEND, FILENOTFND, BLOWTHRU)
