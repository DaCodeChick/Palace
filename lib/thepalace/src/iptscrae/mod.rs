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

// TODO: Implement Iptscrae interpreter
// - Lexer and tokenizer
// - Parser and AST generation
// - Stack-based VM
// - Event system
// - Standard library functions
// - Security sandboxing for cyborg scripts
