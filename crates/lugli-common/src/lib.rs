//! Core types and utilities shared across all Lugli crates.
//!
//! Provides foundational types for runtime values, error handling, source location tracking,
//! string interning, garbage collection infrastructure, and stdlib abstraction.

pub mod error;
pub mod gc;
pub mod span;
pub mod stdlib;
pub mod string_pool;
pub mod value;

pub use error::*;
pub use gc::*;
pub use span::*;
pub use stdlib::*;
pub use string_pool::*;
pub use value::*;
