//! Bytecode virtual machine for the Lugli programming language.
//!
//! # Recommended Usage
//!
//! Use the new unified `Vm` API:
//!
//! ```ignore
//! use lugli_vm::Vm;
//!
//! let mut vm = Vm::new();
//! let result = vm.compile_and_run(&program, span_map)?;
//! ```
//!
//! Or with custom configuration:
//!
//! ```ignore
//! let mut vm = Vm::builder()
//!     .with_source("file.lg".to_string(), source.clone())
//!     .debug(true)
//!     .build();
//! ```

use thiserror::Error;

pub mod builder;
pub mod bytecode;
pub mod compiler;
pub mod debug;
pub mod error_formatter;
pub mod machine;
pub mod module;

// New unified API (recommended)
pub use builder::{Vm, VmBuilder};

// Core VM components
pub use bytecode::{Bytecode, Instruction, SourceLocation};
pub use compiler::Compiler;
pub use error_formatter::{ErrorFormatter, suggest_similar_name};
pub use machine::Machine;
pub use module::{Module, ModuleCache, ModuleResolver};

// Re-export common types for convenience
pub use lugli_common::{ErrorCode, LugliError, SourceContext, Value};

#[derive(Error, Debug)]
pub enum VmError {
    #[error("Compilation error: {0}")]
    CompilationError(String),
    #[error("Runtime error: {0}")]
    RuntimeError(#[from] LugliError),
    #[error("VM error: {0}")]
    VmError(String),
}

pub fn get_bytecode_stats(bytecode: &Bytecode) -> (usize, usize, usize) {
    let instruction_count = bytecode.instructions.len();
    let constant_count = bytecode.constants.len();
    // Rough estimate: 4 bytes per instruction + estimated value sizes
    let estimated_memory = instruction_count * 4 + constant_count * 32; // rough estimate
    (instruction_count, constant_count, estimated_memory)
}
