//! Bytecode virtual machine for the Lugli programming language.

use thiserror::Error;

pub mod bytecode;
pub mod compiler;
pub mod machine;
pub mod operations;

// Core VM components
pub use bytecode::{Bytecode, Instruction};
pub use compiler::Compiler;
pub use machine::VirtualMachine;

// Re-export common types for convenience
pub use lugli_common::{Value, LugliError};

#[derive(Error, Debug)]
pub enum VmError {
    #[error("Compilation error: {0}")]
    CompilationError(String),
    #[error("Runtime error: {0}")]
    RuntimeError(#[from] LugliError),
    #[error("VM error: {0}")]
    VmError(String),
}

pub fn compile(program: &lugli_ast::Program) -> Result<Bytecode, VmError> {
    let mut compiler = Compiler::new();
    compiler.compile(program)
        .map_err(|e| VmError::CompilationError(e.to_string()))
}

pub fn run(bytecode: &Bytecode) -> Result<Value, VmError> {
    let mut vm = VirtualMachine::new();
    vm.run(bytecode).map_err(VmError::RuntimeError)
}

pub fn compile_and_run(program: &lugli_ast::Program) -> Result<Value, VmError> {
    let bytecode = compile(program)?;
    run(&bytecode)
}

pub fn run_with_vm(vm: &mut VirtualMachine, bytecode: &Bytecode) -> Result<Value, VmError> {
    vm.run(bytecode).map_err(VmError::RuntimeError)
}

pub fn get_bytecode_stats(bytecode: &Bytecode) -> (usize, usize, usize) {
    let instruction_count = bytecode.instructions.len();
    let constant_count = bytecode.constants.len();
    // Rough estimate: 4 bytes per instruction + estimated value sizes
    let estimated_memory = instruction_count * 4 + constant_count * 32; // rough estimate
    (instruction_count, constant_count, estimated_memory)
}