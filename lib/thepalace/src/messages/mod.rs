//! Palace Protocol network message types.
//!
//! The Palace Protocol uses a generic message structure:
//! - 4 bytes: Event type (big-endian u32, typically ASCII 4-char code)
//! - 4 bytes: Message length (big-endian u32, excluding header)
//! - 4 bytes: Reference number (big-endian i32)
//! - Variable: Payload data
//!
//! This module will contain implementations for all 60+ Palace Protocol message types
//! including authentication, room navigation, user presence, chat, props, assets, etc.

// TODO: Implement message types
// - MSG_TIYID (0x74697972 'tiyr') - Client version identification
// - MSG_REGI (0x72656769 'regi') - User registration
// - MSG_LOGON (0x6c6f676f 'logo') - User logon
// - MSG_AUTH (0x61757468 'auth') - Authentication response
// - MSG_ROOMGOTO (0x726f6f6d 'room') - Change rooms
// - MSG_USERNEW (0x75736572 'user') - New user entered
// - MSG_CHAT (0x63686174 'chat') - Chat message
// - ... and 50+ more message types
