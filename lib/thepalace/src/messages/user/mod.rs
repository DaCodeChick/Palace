//! User message payloads
//!
//! This module implements message structures for user-related operations:
//! - MessageId::UserNew: New user entering a room
//! - MessageId::UserExit: User leaving a room
//! - MessageId::UserMove: User moving to a new position
//! - MessageId::UserName: User changing their name
//! - MessageId::UserColor: User changing their avatar color
//! - MessageId::UserFace: User changing their face
//! - MessageId::UserProp: User changing their props
//! - MessageId::UserDesc: Bulk user appearance change

mod records;
mod user_ops;

pub use records::UserRec;
pub use user_ops::{
    UserColorMsg, UserDescMsg, UserExitMsg, UserFaceMsg, UserMoveMsg, UserNameMsg, UserNewMsg,
    UserPropMsg,
};
