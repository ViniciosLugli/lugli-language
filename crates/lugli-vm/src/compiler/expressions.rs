use lugli_common::{Value, LugliError};
use lugli_ast::{Expr, LiteralValue};
use crate::{Instruction};
use super::Compiler;

impl Compiler {
    pub(super) fn compile_expr(&mut self, expr: &Expr) -> Result<(), LugliError> {
        match expr {
            Expr::Literal { value, .. } => {
                let constant_index = self.add_constant(self.literal_to_value(value));
                self.emit(Instruction::Constant(constant_index));
                Ok(())
            }
            Expr::Identifier { name, .. } => {
                if let Some(&local_index) = self.locals.get(name) {
                    self.emit(Instruction::Load(local_index));
                    Ok(())
                } else {
                    let name_index = self.add_constant(Value::String(name.clone()));
                    self.emit(Instruction::LoadGlobal(name_index));
                    Ok(())
                }
            }
            Expr::Binary { left, operator, right, .. } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;

                match operator.kind {
                    lugli_lexer::TokenKind::Plus => self.emit(Instruction::Add),
                    lugli_lexer::TokenKind::Minus => self.emit(Instruction::Subtract),
                    lugli_lexer::TokenKind::Star => self.emit(Instruction::Multiply),
                    lugli_lexer::TokenKind::Slash => self.emit(Instruction::Divide),
                    lugli_lexer::TokenKind::Percent => self.emit(Instruction::Modulo),
                    lugli_lexer::TokenKind::EqualEqual => self.emit(Instruction::Equal),
                    lugli_lexer::TokenKind::BangEqual => self.emit(Instruction::NotEqual),
                    lugli_lexer::TokenKind::Greater => self.emit(Instruction::Greater),
                    lugli_lexer::TokenKind::GreaterEqual => self.emit(Instruction::GreaterEqual),
                    lugli_lexer::TokenKind::Less => self.emit(Instruction::Less),
                    lugli_lexer::TokenKind::LessEqual => self.emit(Instruction::LessEqual),
                    lugli_lexer::TokenKind::And => self.emit(Instruction::And),
                    lugli_lexer::TokenKind::Or => self.emit(Instruction::Or),
                    _ => return Err(LugliError::runtime(format!("Unsupported binary operator: {:?}", operator.kind))),
                }
                Ok(())
            }
            Expr::Unary { operator, operand, .. } => {
                self.compile_expr(operand)?;

                match operator.kind {
                    lugli_lexer::TokenKind::Minus => self.emit(Instruction::Negate),
                    lugli_lexer::TokenKind::Bang => self.emit(Instruction::Not),
                    _ => return Err(LugliError::runtime(format!("Unsupported unary operator: {:?}", operator.kind))),
                }
                Ok(())
            }
            Expr::Call { callee, arguments, .. } => {
                // Compile arguments first (they'll be on stack in order)
                for arg in arguments {
                    self.compile_expr(arg)?;
                }

                // Compile the function itself
                self.compile_expr(callee)?;

                // Emit call instruction with argument count
                self.emit(Instruction::Call(arguments.len() as u8));
                Ok(())
            }
            Expr::Get { object, name, .. } => {
                // Check if this is a global variable access (global.variable)
                if let Expr::Identifier { name: obj_name, .. } = object.as_ref() {
                    if obj_name == "global" {
                        // This is a variable access, not property access
                        let name_index = self.add_constant(Value::String(name.clone()));
                        self.emit(Instruction::LoadGlobal(name_index));
                        return Ok(());
                    }
                }

                // Regular property access
                // Compile the object expression
                self.compile_expr(object)?;

                // Add property name as constant and emit GetProperty instruction
                let name_index = self.add_constant(Value::String(name.clone()));
                self.emit(Instruction::GetProperty(name_index));
                Ok(())
            }
            Expr::Set { object, name, value, .. } => {
                // Check if this is a variable assignment
                if let Expr::Identifier { name: obj_name, .. } = object.as_ref() {
                    if obj_name == "global" {
                        // This is a variable assignment - either global or local
                        self.compile_expr(value)?;

                        // Check if it's a local variable first
                        if let Some(&local_index) = self.locals.get(name) {
                            // Local variable assignment
                            self.emit(Instruction::Store(local_index));
                            // Load it back to leave on stack for expression result
                            self.emit(Instruction::Load(local_index));
                        } else {
                            // Global variable assignment
                            let name_index = self.add_constant(Value::String(name.clone()));
                            self.emit(Instruction::StoreGlobal(name_index));
                        }
                        return Ok(());
                    }
                }

                // Regular property assignment
                // Compile the value first (it will be consumed by SetProperty)
                self.compile_expr(value)?;

                // Compile the object expression
                self.compile_expr(object)?;

                // Add property name as constant and emit SetProperty instruction
                let name_index = self.add_constant(Value::String(name.clone()));
                self.emit(Instruction::SetProperty(name_index));

                // If object is a simple identifier (local variable), store the modified object back
                if let Expr::Identifier { name: var_name, .. } = object.as_ref() {
                    if let Some(&local_index) = self.locals.get(var_name) {
                        self.emit(Instruction::Store(local_index));
                        // Load it back to leave on stack for expression result
                        self.emit(Instruction::Load(local_index));
                    }
                }

                Ok(())
            }
            Expr::List { elements, .. } => {
                // Compile all elements (they'll be on stack in order)
                for element in elements {
                    self.compile_expr(element)?;
                }

                // Emit MakeList instruction with element count
                self.emit(Instruction::MakeList(elements.len()));
                Ok(())
            }
            Expr::Dict { pairs, .. } => {
                // Compile all key-value pairs (keys and values will be on stack alternating)
                for (key, value) in pairs {
                    self.compile_expr(key)?;
                    self.compile_expr(value)?;
                }

                // Emit MakeDict instruction with pair count
                self.emit(Instruction::MakeDict(pairs.len()));
                Ok(())
            }
            _ => Err(LugliError::runtime("Expression compilation not yet implemented")),
        }
    }

    pub(super) fn literal_to_value(&self, literal: &LiteralValue) -> Value {
        match literal {
            LiteralValue::Number(n) => Value::Number(*n),
            LiteralValue::String(s) => Value::String(s.clone()),
            LiteralValue::Boolean(b) => Value::Bool(*b),
            LiteralValue::Null => Value::Null,
        }
    }
}