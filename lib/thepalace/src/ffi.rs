//! Foreign Function Interface (FFI) bindings for C/C++ interoperability.
//!
//! This module provides C-compatible API bindings for the Palace Protocol library,
//! allowing the C++ Qt client to use the Rust implementation of the protocol.
//!
//! ## C API Design
//!
//! - Opaque pointers for Rust types
//! - C-compatible function signatures
//! - Manual memory management via create/destroy functions
//! - Error handling via result codes
//!
//! ## Code Generation
//!
//! C headers are automatically generated using `cbindgen` from these FFI functions.

// TODO: Implement FFI bindings
// - Opaque handle types
// - Message parsing/serialization functions
// - Connection management
// - Error handling
// - Generate C headers with cbindgen
