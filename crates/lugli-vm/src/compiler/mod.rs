use crate::{Bytecode, Instruction};
use hashbrown::HashMap;
use lugli_ast::{Stmt, SpanMap};
use lugli_common::{LugliError, Value};

mod control_flow;
mod expressions;
mod optimizer;
mod statements;

use optimizer::PeepholeOptimizer;

pub struct CompilerDebug {
    pub trace_emit: bool,
    pub trace_locals: bool,
    pub trace_constants: bool,
    pub instruction_count: usize,
}

impl Default for CompilerDebug {
    fn default() -> Self {
        Self {
            trace_emit: std::env::var("LUGLI_TRACE_COMPILE").is_ok(),
            trace_locals: std::env::var("LUGLI_TRACE_LOCALS").is_ok(),
            trace_constants: std::env::var("LUGLI_TRACE_CONSTANTS").is_ok(),
            instruction_count: 0,
        }
    }
}

pub struct Compiler {
    bytecode: Bytecode,
    locals: HashMap<String, usize>,
    local_count: usize,
    debug: CompilerDebug,
    loop_stack: Vec<(usize, usize, Vec<usize>, Vec<usize>)>, // (loop_start, continue_target, continue_jumps, break_jumps)
    loop_depth: usize,                                       // For unique loop variable names
    scope_depth: usize,                                      // 0 = global scope, >0 = function/local scope
    upvalues: HashMap<String, usize>,                        // Map of upvalue names to indices (for closures)
    upvalue_count: usize,                                    // Number of upvalues in current closure
    file_path: String,                                       // Source file path for error reporting
    source_code: Option<String>,                             // Source code for error context
    span_map: SpanMap,                                       // Map from NodeId to Span
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
            locals: HashMap::new(),
            local_count: 0,
            debug: CompilerDebug::default(),
            loop_stack: Vec::new(),
            loop_depth: 0,
            scope_depth: 0, // Start at global scope
            upvalues: HashMap::new(),
            upvalue_count: 0,
            file_path: "<unknown>".to_string(),
            source_code: None,
            span_map: SpanMap::new(),
        }
    }

    pub fn with_source(file_path: String, source_code: String) -> Self {
        let mut compiler = Self::new();
        compiler.file_path = file_path.clone();
        compiler.source_code = Some(source_code.clone());
        compiler.bytecode = Bytecode::with_source(source_code);
        compiler.span_map = SpanMap::new();
        compiler
    }

    pub fn compile(&mut self, program: &lugli_ast::Program, span_map: SpanMap) -> Result<Bytecode, LugliError> {
        self.span_map = span_map;
        let stmt_count = program.statements.len();

        for (i, stmt) in program.statements.iter().enumerate() {
            self.compile_stmt(stmt)?;

            // Pop intermediate expression results except for the last statement
            if i < stmt_count - 1 && matches!(stmt, lugli_ast::Stmt::Expression { .. }) {
                self.emit_unknown(Instruction::Pop);
            }
        }

        // Ensure there's always a return instruction at the end
        if self.bytecode.instructions.is_empty() || !matches!(self.bytecode.instructions.last(), Some(Instruction::Return)) {
            // If the last statement wasn't an expression or return, add null
            if stmt_count == 0
                || !matches!(program.statements.last(), Some(lugli_ast::Stmt::Expression { .. }) | Some(lugli_ast::Stmt::Return { .. }))
            {
                let null_index = self.add_constant(Value::Null);
                self.emit_unknown(Instruction::Constant(null_index));
            }
            self.emit_unknown(Instruction::Return);
        }

        // Apply peephole optimizations if enabled
        if std::env::var("LUGLI_NO_OPTIMIZE").is_err() {
            let optimizer = PeepholeOptimizer::new();
            self.bytecode.instructions = optimizer.optimize(std::mem::take(&mut self.bytecode.instructions));
        }

        Ok(std::mem::take(&mut self.bytecode))
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
    pub(crate) fn add_constant(&mut self, value: Value) -> usize { self.bytecode.add_constant(value) }

    pub(crate) fn emit_unknown(&mut self, instruction: Instruction) {
        if self.debug.trace_emit {
            eprintln!("[COMPILE] Emit {:04}: {:?}", self.bytecode.instructions.len(), instruction);
        }
        self.debug.instruction_count += 1;
        self.bytecode.emit(instruction);
    }

    pub(crate) fn declare_local(&mut self, name: String) -> usize {
        let index = self.local_count;
        if self.debug.trace_locals {
            eprintln!("[COMPILE] Declare local '{}' at index {}", name, index);
        }
        self.locals.insert(name, index);
        self.local_count += 1;
        index
    }

    pub(crate) fn current_instruction(&self) -> usize { self.bytecode.instructions.len() }

    pub(crate) fn emit_jump(&mut self, instruction: Instruction) -> usize {
        let index = self.bytecode.instructions.len();
        self.emit_unknown(instruction);
        index
    }

    pub(crate) fn patch_jump(&mut self, jump_index: usize) -> Result<(), LugliError> {
        let current_address = self.bytecode.instructions.len();

        match &mut self.bytecode.instructions.get_mut(jump_index) {
            Some(Instruction::Jump(addr)) => {
                *addr = current_address;
                Ok(())
            }
            Some(Instruction::JumpIfFalse(addr)) => {
                *addr = current_address;
                Ok(())
            }
            Some(_) => Err(LugliError::runtime("Internal compiler error: Expected jump instruction for patching")),
            None => Err(LugliError::runtime("Internal compiler error: Invalid jump index")),
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    pub(crate) fn try_evaluate_constant(&self, expr: &lugli_ast::Expr) -> Option<Value> {
        use lugli_ast::{Expr, LiteralValue};
        match expr {
            Expr::Literal {
                value, ..
            } => Some(match value {
                LiteralValue::Number(n) => Value::Number(*n),
                LiteralValue::String(s) => {
                    let id = {
                        let mut pool = self.bytecode.string_pool.borrow_mut();
                        pool.intern(s)
                    };
                    Value::String(id)
                }
                LiteralValue::Boolean(b) => Value::Bool(*b),
                LiteralValue::Null => Value::Null,
            }),
            Expr::List {
                elements, ..
            } => {
                let mut const_elements = Vec::new();
                for elem in elements {
                    const_elements.push(self.try_evaluate_constant(elem)?);
                }
                Some(Value::List(std::rc::Rc::new(std::cell::RefCell::new(const_elements))))
            }
            Expr::Dict {
                pairs, ..
            } => {
                let mut const_dict = hashbrown::HashMap::new();
                for (key_expr, value_expr) in pairs {
                    if let Expr::Literal {
                        value: LiteralValue::String(key), ..
                    } = key_expr
                    {
                        // Intern the key first, then drop the borrow before recursing
                        let key_id = {
                            let mut pool = self.bytecode.string_pool.borrow_mut();
                            pool.intern(key)
                        };
                        const_dict.insert(key_id, self.try_evaluate_constant(value_expr)?);
                    } else {
                        return None;
                    }
                }
                Some(Value::Dict(std::rc::Rc::new(std::cell::RefCell::new(const_dict))))
            }
            _ => None,
        }
    }
}

impl Default for Compiler {
    fn default() -> Self { Self::new() }
}
