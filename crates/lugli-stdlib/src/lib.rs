//! Standard library implementation for Lugli.
//!
//! Provides native functions and method implementations for built-in types (List, Dict, String).
//! Functions are registered with the VM and callable from Lugli code.

use lugli_common::{LugliError, StringPool, Value};

pub mod core;
mod helpers;
pub mod io;
pub mod methods;
pub mod time;

pub use methods::MethodRegistry;

// Re-export individual functions to avoid ambiguous glob re-exports
// Main access point is get_global_functions() which combines all modules

pub type NativeFunction = fn(&[Value], &mut StringPool) -> Result<Value, LugliError>;

pub fn get_global_functions() -> Vec<(&'static str, NativeFunction)> {
    let mut functions = Vec::new();
    functions.extend(core::get_functions());
    functions.extend(io::get_functions());
    functions.extend(time::get_functions());
    functions
}
