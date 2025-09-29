use lugli_common::{Value, LugliError};
use lugli_ast::Stmt;
use crate::{Bytecode, Instruction};
use hashbrown::HashMap;

mod expressions;
mod statements;
mod control_flow;

pub struct Compiler {
    bytecode: Bytecode,
    locals: HashMap<String, usize>,
    local_count: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
            locals: HashMap::new(),
            local_count: 0,
        }
    }

    pub fn compile(&mut self, program: &lugli_ast::Program) -> Result<Bytecode, LugliError> {
        let stmt_count = program.statements.len();

        for (i, stmt) in program.statements.iter().enumerate() {
            self.compile_stmt(stmt)?;

            // Pop intermediate expression results except for the last statement
            if i < stmt_count - 1 {
                if matches!(stmt, lugli_ast::Stmt::Expression { .. }) {
                    self.emit(Instruction::Pop);
                }
            }
        }

        // Ensure there's always a return instruction at the end
        if self.bytecode.instructions.is_empty() ||
           !matches!(self.bytecode.instructions.last(), Some(Instruction::Return)) {
            // If the last statement wasn't an expression or return, add null
            if stmt_count == 0 || !matches!(program.statements.last(),
                Some(lugli_ast::Stmt::Expression { .. }) | Some(lugli_ast::Stmt::Return { .. })) {
                let null_index = self.add_constant(Value::Null);
                self.emit(Instruction::Constant(null_index));
            }
            self.emit(Instruction::Return);
        }

        Ok(self.bytecode.clone())
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), LugliError> {
        // Try simple statements first
        if self.compile_simple_stmt(stmt)? {
            return Ok(());
        }

        // Try control flow statements
        if self.compile_control_flow_stmt(stmt)? {
            return Ok(());
        }

        // If neither handled it, return error
        Err(LugliError::runtime("Statement compilation not yet implemented"))
    }

    // Helper methods used by submodules
    pub(crate) fn add_constant(&mut self, value: Value) -> usize {
        self.bytecode.add_constant(value)
    }

    pub(crate) fn emit(&mut self, instruction: Instruction) {
        self.bytecode.emit(instruction);
    }

    pub(crate) fn declare_local(&mut self, name: String) -> usize {
        let index = self.local_count;
        self.locals.insert(name, index);
        self.local_count += 1;
        index
    }

    pub(crate) fn current_instruction(&self) -> usize {
        self.bytecode.instructions.len()
    }

    pub(crate) fn emit_jump(&mut self, instruction: Instruction) -> usize {
        let index = self.bytecode.instructions.len();
        self.emit(instruction);
        index
    }

    pub(crate) fn patch_jump(&mut self, jump_index: usize) {
        let current_address = self.bytecode.instructions.len();

        match &mut self.bytecode.instructions[jump_index] {
            Instruction::Jump(addr) => *addr = current_address,
            Instruction::JumpIfFalse(addr) => *addr = current_address,
            _ => panic!("Expected jump instruction"),
        }
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}