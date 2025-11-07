//! Standard library implementation for Lugli.
//!
//! Provides native functions and method implementations for built-in types (List, Dict, String).
//! Functions are registered with the VM and callable from Lugli code.

use lugli_common::{
    LugliError, StringPool, Value,
    stdlib::{MethodRegistry as MethodRegistryTrait, StandardLibrary, StandardLibraryFactory},
};

pub mod core;
pub mod io;
pub mod methods;
pub mod time;
pub mod validation;

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

/// Default Lugli standard library implementation
pub struct LugliStdlib;

impl StandardLibrary for LugliStdlib {
    fn get_global_functions(&self) -> Vec<(&'static str, NativeFunction)> { get_global_functions() }

    fn get_method_registry(&self) -> Box<dyn MethodRegistryTrait> { Box::new(MethodRegistry::new()) }

    fn info(&self) -> (&'static str, &'static str) { ("lugli-stdlib", env!("CARGO_PKG_VERSION")) }
}

impl StandardLibraryFactory for LugliStdlib {
    fn create() -> Box<dyn StandardLibrary> { Box::new(LugliStdlib) }
}
