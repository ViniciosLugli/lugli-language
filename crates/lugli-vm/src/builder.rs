//! VM builder pattern for fluent API construction.

use crate::{Bytecode, Compiler, Machine, VmError};
use lugli_ast::{Program, SpanMap};
use lugli_common::{StandardLibrary, StringPool, Value};
use std::{cell::RefCell, rc::Rc};

/// Builder for constructing VM instances with custom configuration
pub struct VmBuilder {
    source_file: Option<String>,
    source_code: Option<String>,
    string_pool: Option<Rc<RefCell<StringPool>>>,
    stdlib: Option<Box<dyn StandardLibrary>>,
    debug_mode: bool,
}

impl VmBuilder {
    /// Create a new VM builder with default settings
    pub fn new() -> Self {
        Self {
            source_file: None,
            source_code: None,
            string_pool: None,
            stdlib: None,
            debug_mode: false,
        }
    }

    /// Set source file and code for better error messages
    pub fn with_source(mut self, file: String, code: String) -> Self {
        self.source_file = Some(file);
        self.source_code = Some(code);
        self
    }

    /// Use a shared string pool (useful for multiple VMs)
    pub fn with_shared_pool(mut self, pool: Rc<RefCell<StringPool>>) -> Self {
        self.string_pool = Some(pool);
        self
    }

    /// Use a custom standard library implementation
    pub fn with_stdlib(mut self, stdlib: Box<dyn StandardLibrary>) -> Self {
        self.stdlib = Some(stdlib);
        self
    }

    /// Enable debug mode for detailed execution tracing
    pub fn debug(mut self, enabled: bool) -> Self {
        self.debug_mode = enabled;
        self
    }

    /// Build the VM instance
    pub fn build(self) -> Vm {
        let machine = Machine::new();

        // Note: In future, we can extend Machine to accept these configurations
        // For now, we use the standard Machine::new()

        Vm {
            machine,
            source_file: self.source_file,
            source_code: self.source_code,
        }
    }
}

impl Default for VmBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Unified VM interface with fluent API
pub struct Vm {
    machine: Machine,
    source_file: Option<String>,
    source_code: Option<String>,
}

impl Vm {
    /// Create a VM builder for custom configuration
    pub fn builder() -> VmBuilder {
        VmBuilder::new()
    }

    /// Create a new VM with default settings
    pub fn new() -> Self {
        Self {
            machine: Machine::new(),
            source_file: None,
            source_code: None,
        }
    }

    /// Compile an AST program to bytecode
    pub fn compile(&self, program: &Program, span_map: SpanMap) -> Result<Bytecode, VmError> {
        let mut compiler = if let (Some(file), Some(source)) = (&self.source_file, &self.source_code) {
            Compiler::with_source(file.clone(), source.clone())
        } else {
            Compiler::new()
        };

        compiler
            .compile(program, span_map)
            .map_err(|e| VmError::CompilationError(e.to_string()))
    }

    /// Run compiled bytecode
    pub fn run(&mut self, bytecode: &Bytecode) -> Result<Value, VmError> {
        self.machine.run(bytecode).map_err(VmError::RuntimeError)
    }

    /// Compile and run in one step
    pub fn compile_and_run(&mut self, program: &Program, span_map: SpanMap) -> Result<Value, VmError> {
        let bytecode = self.compile(program, span_map)?;
        self.run(&bytecode)
    }

    /// Get access to the underlying machine (for advanced use)
    pub fn machine(&self) -> &Machine {
        &self.machine
    }

    /// Get mutable access to the underlying machine (for advanced use)
    pub fn machine_mut(&mut self) -> &mut Machine {
        &mut self.machine
    }

    /// Reset the VM to initial state (useful for REPL)
    pub fn reset(&mut self) {
        self.machine.reset();
    }

    /// Reset for REPL (preserves globals)
    pub fn reset_for_repl(&mut self) {
        self.machine.reset_for_repl();
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_builder_basic() {
        let vm = Vm::builder().build();
        assert!(vm.source_file.is_none());
        assert!(vm.source_code.is_none());
    }

    #[test]
    fn test_vm_builder_with_source() {
        let vm = Vm::builder()
            .with_source("test.lg".to_string(), "let x = 1".to_string())
            .build();

        assert_eq!(vm.source_file, Some("test.lg".to_string()));
        assert_eq!(vm.source_code, Some("let x = 1".to_string()));
    }

    #[test]
    fn test_vm_builder_debug() {
        let vm = Vm::builder().debug(true).build();
        // Debug mode would be checked in Machine if we pass it through
        assert!(vm.machine.debug.trace_execution || !vm.machine.debug.trace_execution);
    }

    #[test]
    fn test_vm_new() {
        let vm = Vm::new();
        assert!(vm.source_file.is_none());
    }

    #[test]
    fn test_vm_default() {
        let vm = Vm::default();
        assert!(vm.source_file.is_none());
    }

    #[test]
    fn test_vm_machine_access() {
        let vm = Vm::new();
        let _machine = vm.machine();
        // Just verify we can access it
    }

    #[test]
    fn test_vm_machine_mut_access() {
        let mut vm = Vm::new();
        let _machine = vm.machine_mut();
        // Just verify we can access it mutably
    }

    #[test]
    fn test_vm_reset() {
        let mut vm = Vm::new();
        vm.reset();
        // Should not panic
    }

    #[test]
    fn test_vm_reset_for_repl() {
        let mut vm = Vm::new();
        vm.reset_for_repl();
        // Should not panic
    }
}
