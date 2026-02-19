//! Iptscrae builtin function implementations.
//!
//! This module contains the implementations of all Iptscrae builtin functions,
//! organized by category for better maintainability.

mod stack;
mod string;
mod math;
mod logic;
mod array;
mod palace;

pub use stack::execute_stack_builtin;
pub use string::execute_string_builtin;
pub use math::execute_math_builtin;
pub use logic::execute_logic_builtin;
pub use array::execute_array_builtin;
pub use palace::execute_palace_builtin;
