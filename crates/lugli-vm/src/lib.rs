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

// Legacy API functions (maintained for backward compatibility)
// Use Vm API instead for new code

#[deprecated(since = "0.5.0", note = "Use `Vm::new().compile()` instead")]
pub fn compile(program: &lugli_ast::Program, span_map: lugli_ast::SpanMap) -> Result<Bytecode, VmError> {
    let vm = Vm::new();
    vm.compile(program, span_map)
}

#[deprecated(since = "0.5.0", note = "Use `Vm::builder().with_source().build().compile()` instead")]
pub fn compile_with_source(program: &lugli_ast::Program, span_map: lugli_ast::SpanMap, file_path: &str, source: &str) -> Result<Bytecode, VmError> {
    let vm = Vm::builder()
        .with_source(file_path.to_string(), source.to_string())
        .build();
    vm.compile(program, span_map)
}

#[deprecated(since = "0.5.0", note = "Use `Vm::new().run()` instead")]
pub fn run(bytecode: &Bytecode) -> Result<Value, VmError> {
    let mut vm = Vm::new();
    vm.run(bytecode)
}

#[deprecated(since = "0.5.0", note = "Use `Vm::new().compile_and_run()` instead")]
pub fn compile_and_run(program: &lugli_ast::Program, span_map: lugli_ast::SpanMap) -> Result<Value, VmError> {
    let mut vm = Vm::new();
    vm.compile_and_run(program, span_map)
}

#[deprecated(since = "0.5.0", note = "Use `Vm::builder().with_source().build().compile_and_run()` instead")]
pub fn compile_and_run_with_source(
    program: &lugli_ast::Program,
    span_map: lugli_ast::SpanMap,
    file_path: &str,
    source: &str,
) -> Result<Value, VmError> {
    let mut vm = Vm::builder()
        .with_source(file_path.to_string(), source.to_string())
        .build();
    vm.compile_and_run(program, span_map)
}

#[deprecated(since = "0.5.0", note = "Use `Vm::machine_mut()` and call `.run()` directly")]
pub fn run_with_vm(vm: &mut Machine, bytecode: &Bytecode) -> Result<Value, VmError> {
    vm.run(bytecode).map_err(VmError::RuntimeError)
}

pub fn get_bytecode_stats(bytecode: &Bytecode) -> (usize, usize, usize) {
    let instruction_count = bytecode.instructions.len();
    let constant_count = bytecode.constants.len();
    // Rough estimate: 4 bytes per instruction + estimated value sizes
    let estimated_memory = instruction_count * 4 + constant_count * 32; // rough estimate
    (instruction_count, constant_count, estimated_memory)
}
