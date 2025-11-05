//! Core types and utilities shared across all Lugli crates.
//!
//! Provides foundational types for runtime values, error handling, source location tracking,
//! string interning, and garbage collection infrastructure.

pub mod error;
pub mod gc;
pub mod span;
pub mod string_pool;
pub mod value;

pub use error::*;
pub use gc::*;
pub use span::*;
pub use string_pool::*;
pub use value::*;
