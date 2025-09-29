use lugli_common::{Value, LugliError};
use lugli_ast::Stmt;
use crate::Instruction;
use super::Compiler;

impl Compiler {
    pub(super) fn compile_simple_stmt(&mut self, stmt: &Stmt) -> Result<bool, LugliError> {
        match stmt {
            Stmt::Expression { expr, .. } => {
                self.compile_expr(expr)?;
                // Keep expression result on stack for return value
                Ok(true) // Handled
            }
            Stmt::VarDecl { name, initializer, .. } => {
                let local_index = self.declare_local(name.clone());

                if let Some(init) = initializer {
                    self.compile_expr(init)?;
                } else {
                    let null_index = self.add_constant(Value::Null);
                    self.emit(Instruction::Constant(null_index));
                }

                self.emit(Instruction::Store(local_index));
                Ok(true) // Handled
            }
            Stmt::Return { value, .. } => {
                if let Some(val) = value {
                    self.compile_expr(val)?;
                } else {
                    let null_index = self.add_constant(Value::Null);
                    self.emit(Instruction::Constant(null_index));
                }
                self.emit(Instruction::Return);
                Ok(true) // Handled
            }
            Stmt::FnDecl { name, params, body, .. } => {
                let jump_over_body = self.emit_jump(Instruction::Jump(0));

                let body_start = self.current_instruction();

                let saved_locals = self.locals.clone();
                let saved_local_count = self.local_count;

                self.locals.clear();
                self.local_count = 0;

                for param in params {
                    self.declare_local(param.clone());
                }

                for stmt in body {
                    self.compile_stmt(stmt)?;
                }

                if !matches!(body.last(), Some(Stmt::Return { .. })) {
                    let null_index = self.add_constant(Value::Null);
                    self.emit(Instruction::Constant(null_index));
                    self.emit(Instruction::Return);
                }

                self.patch_jump(jump_over_body);

                self.locals = saved_locals;
                self.local_count = saved_local_count;

                let function_value = Value::Function {
                    name: name.clone(),
                    params: params.clone(),
                    arity: params.len(),
                    body_start,
                };

                let function_index = self.add_constant(function_value);
                self.emit(Instruction::DefineFunction(function_index));

                let name_index = self.add_constant(Value::String(name.clone()));
                self.emit(Instruction::StoreGlobal(name_index));

                Ok(true) // Handled
            }
            _ => Ok(false) // Not handled - control flow statements go to control_flow.rs
        }
    }
}