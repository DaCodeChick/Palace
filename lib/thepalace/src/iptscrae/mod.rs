//! Iptscrae scripting language interpreter.
//!
//! Iptscrae is a stack-based scripting language used throughout the Palace Protocol
//! for room scripts, prop behaviors, and cyborg (client-side bot) scripts.
//!
//! ## Language Features
//!
//! - Stack-based execution model
//! - 20+ event types (SELECT, ENTER, LEAVE, INCHAT, PROPCHANGE, etc.)
//! - Extensive standard library for Palace operations
//! - Support for conditionals, loops, and subroutines
//!
//! ## Security Model
//!
//! - **Server scripts**: Full trust, no sandboxing
//! - **Cyborg scripts**: Sandboxed with instruction limits and timeouts
//! - Room flag RF_CyborgFreeZone (0x1000) disables cyborg scripts
//! - Server flag SF_AllowCyborgs (0x0200) enables globally

pub mod ast;
pub mod builtins;
pub mod context;
pub mod events;
pub mod lexer;
pub mod parser;
#[cfg(feature = "room-script")]
pub mod room_script;
#[cfg(feature = "room-script")]
pub mod room_script_parser;
pub mod token;
pub mod value;
pub mod vm;

pub use ast::{BinOp, Block, EventHandler, Expr, Script, Statement, UnaryOp};
pub use context::{ScriptActions, ScriptContext, SecurityLevel};
pub use events::{EventMask, EventType};
pub use lexer::{LexError, Lexer};
pub use parser::{ParseError, Parser};
#[cfg(feature = "room-script")]
pub use room_script::{DoorDecl, PictureDecl, RoomDecl, RoomFlags, SpotDecl, StateDecl};
#[cfg(feature = "room-script")]
pub use room_script_parser::RoomScriptParser;
pub use token::{SourcePos, Token, TokenKind};
pub use value::Value;
pub use vm::{ExecutionLimits, Vm, VmError};
