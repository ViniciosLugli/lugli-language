use lugli_common::{Value, LugliError};

pub mod core;
pub mod io;
pub mod time;

// Re-export individual functions to avoid ambiguous glob re-exports
// Main access point is get_global_functions() which combines all modules

pub type NativeFunction = fn(&[Value]) -> Result<Value, LugliError>;

pub fn get_global_functions() -> Vec<(&'static str, NativeFunction)> {
    let mut functions = Vec::new();
    functions.extend(core::get_functions());
    functions.extend(io::get_functions());
    functions.extend(time::get_functions());
    functions
}