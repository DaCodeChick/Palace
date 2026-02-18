//! Room message payloads
//!
//! This module implements message structures for room-related operations:
//! - MessageId::RoomGoto: Client requests to move to a different room
//! - MessageId::RoomDesc: Server describes a room (complex structure with RoomRec)
//! - MessageId::RoomDescEnd: Marks end of room description sequence
//! - MessageId::RoomNew: Create a new room
//! - MessageId::RoomSetDesc: Update room description
//!
//! RoomRec is a complex structure with variable-length data including hotspots,
//! pictures, loose props, draw commands, and embedded strings.

// Sub-modules
mod door_ops;
mod hotspot_ops;
mod picture_ops;
mod prop_ops;
mod records;
mod room_ops;

// Re-export all public items from records
pub use records::{Hotspot, LPropRec, PictureRec, RoomRec};

// Re-export all public items from room_ops
pub use room_ops::{RoomDescEndMsg, RoomDescMsg, RoomGotoMsg};

// Re-export all public items from prop_ops
pub use prop_ops::{ListOfAllRoomsMsg, PropDelMsg, PropMoveMsg, PropNewMsg, RoomListRec};

// Re-export all public items from hotspot_ops
pub use hotspot_ops::{SpotDelMsg, SpotMoveMsg, SpotNewMsg, SpotStateMsg};

// Re-export all public items from door_ops
pub use door_ops::{DoorLockMsg, DoorUnlockMsg};

// Re-export all public items from picture_ops
pub use picture_ops::PictMoveMsg;
