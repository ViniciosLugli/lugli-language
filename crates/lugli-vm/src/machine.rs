use lugli_common::{Value, LugliError};
use crate::{Bytecode, Instruction};
use crate::operations::{apply_binary_op, apply_unary_op};
use lugli_lexer::TokenKind;
use hashbrown::HashMap;

pub struct VirtualMachine {
    stack: Vec<Value>,
    locals: Vec<Value>,
    globals: HashMap<String, Value>,
    ip: usize,
    native_functions: HashMap<String, lugli_stdlib::NativeFunction>,
}

impl VirtualMachine {
    pub fn new() -> Self {
        // Create VM with default stdlib functions
        Self::with_stdlib(lugli_stdlib::get_global_functions())
    }

    pub fn with_stdlib(functions: Vec<(&'static str, lugli_stdlib::NativeFunction)>) -> Self {
        let native_functions = functions.into_iter()
            .map(|(name, func)| (name.to_string(), func))
            .collect();

        Self {
            stack: Vec::new(),
            locals: Vec::new(),
            globals: HashMap::new(),
            ip: 0,
            native_functions,
        }
    }

    pub fn run(&mut self, bytecode: &Bytecode) -> Result<Value, LugliError> {
        self.ip = 0;
        self.stack.clear();
        self.locals.clear();
        self.globals.clear();

        while self.ip < bytecode.instructions.len() {
            match &bytecode.instructions[self.ip] {
                // Stack operations
                Instruction::Constant(index) => {
                    let value = bytecode.constants[*index].clone();
                    self.stack.push(value);
                }
                Instruction::Pop => {
                    self.pop()?;
                }

                // Variable operations
                Instruction::Load(index) => {
                    self.ensure_local_exists(*index);
                    let value = self.locals[*index].clone();
                    self.stack.push(value);
                }
                Instruction::Store(index) => {
                    let value = self.pop()?;
                    self.ensure_local_exists(*index);
                    self.locals[*index] = value;
                }

                // Arithmetic operations
                Instruction::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = apply_binary_op(&a, &TokenKind::Plus, &b)?;
                    self.stack.push(result);
                }
                Instruction::Subtract => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = apply_binary_op(&a, &TokenKind::Minus, &b)?;
                    self.stack.push(result);
                }
                Instruction::Multiply => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = apply_binary_op(&a, &TokenKind::Star, &b)?;
                    self.stack.push(result);
                }
                Instruction::Divide => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = apply_binary_op(&a, &TokenKind::Slash, &b)?;
                    self.stack.push(result);
                }
                Instruction::Modulo => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = apply_binary_op(&a, &TokenKind::Percent, &b)?;
                    self.stack.push(result);
                }

                // Unary operations
                Instruction::Negate => {
                    let operand = self.pop()?;
                    let result = apply_unary_op(&TokenKind::Minus, &operand)?;
                    self.stack.push(result);
                }
                Instruction::Not => {
                    let operand = self.pop()?;
                    let result = apply_unary_op(&TokenKind::Bang, &operand)?;
                    self.stack.push(result);
                }

                // Comparison operations
                Instruction::Equal => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = apply_binary_op(&a, &TokenKind::EqualEqual, &b)?;
                    self.stack.push(result);
                }
                Instruction::Greater => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = apply_binary_op(&a, &TokenKind::Greater, &b)?;
                    self.stack.push(result);
                }
                Instruction::Less => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = apply_binary_op(&a, &TokenKind::Less, &b)?;
                    self.stack.push(result);
                }
                Instruction::NotEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = apply_binary_op(&a, &TokenKind::BangEqual, &b)?;
                    self.stack.push(result);
                }
                Instruction::GreaterEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = apply_binary_op(&a, &TokenKind::GreaterEqual, &b)?;
                    self.stack.push(result);
                }
                Instruction::LessEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = apply_binary_op(&a, &TokenKind::LessEqual, &b)?;
                    self.stack.push(result);
                }
                Instruction::And => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = apply_binary_op(&a, &TokenKind::And, &b)?;
                    self.stack.push(result);
                }
                Instruction::Or => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = apply_binary_op(&a, &TokenKind::Or, &b)?;
                    self.stack.push(result);
                }

                // Control flow
                Instruction::Jump(offset) => {
                    self.ip = *offset;
                    continue;
                }
                Instruction::JumpIfFalse(offset) => {
                    let condition = self.pop()?;
                    if !condition.is_truthy() {
                        self.ip = *offset;
                        continue;
                    }
                }
                Instruction::Loop(offset) => {
                    self.ip = *offset;
                    continue;
                }

                // Function operations
                Instruction::Call(arg_count) => {
                    let args: Vec<Value> = (0..*arg_count)
                        .map(|_| self.pop())
                        .collect::<Result<Vec<_>, _>>()?
                        .into_iter()
                        .rev()
                        .collect();

                    let function = self.pop()?;
                    match function {
                        Value::NativeFunction { name, .. } => {
                            if let Some(func) = self.native_functions.get(&name) {
                                let result = func(&args)?;
                                self.stack.push(result);
                            } else {
                                return Err(LugliError::runtime(format!("Unknown native function: {}", name)));
                            }
                        }
                        Value::Function { arity, body_start, .. } => {
                            if args.len() != arity {
                                return Err(LugliError::runtime(format!("Function expects {} arguments, got {}", arity, args.len())));
                            }

                            return Err(LugliError::runtime("User-defined function calls not yet implemented"));
                        }
                        _ => return Err(LugliError::runtime(format!("Not callable: {}", function.type_name()))),
                    }
                }
                Instruction::Return => {
                    return Ok(self.pop().unwrap_or(Value::Null));
                }

                // I/O operations
                Instruction::Print => {
                    let value = self.pop()?;
                    println!("{}", value);
                }

                // Property operations
                Instruction::GetProperty(name_index) => {
                    let object = self.pop()?;
                    let property_name = &bytecode.constants[*name_index];

                    if let Value::String(name) = property_name {
                        match object {
                            Value::Dict(dict) => {
                                if let Some(value) = dict.get(name) {
                                    self.stack.push(value.clone());
                                } else {
                                    self.stack.push(Value::Null);
                                }
                            }
                            Value::Struct { fields, .. } => {
                                if let Some(value) = fields.get(name) {
                                    self.stack.push(value.clone());
                                } else {
                                    return Err(LugliError::runtime(format!("Property '{}' not found", name)));
                                }
                            }
                            _ => {
                                return Err(LugliError::runtime(format!("Cannot access property '{}' on {}", name, object.type_name())));
                            }
                        }
                    } else {
                        return Err(LugliError::runtime("Property name must be a string"));
                    }
                }

                Instruction::SetProperty(name_index) => {
                    let object = self.pop()?;
                    let value = self.pop()?;
                    let property_name = &bytecode.constants[*name_index];

                    if let Value::String(name) = property_name {
                        match object {
                            Value::Dict(mut dict) => {
                                dict.insert(name.clone(), value);
                                self.stack.push(Value::Dict(dict));
                            }
                            Value::Struct { name: struct_name, mut fields } => {
                                fields.insert(name.clone(), value.clone());
                                self.stack.push(Value::Struct { name: struct_name, fields });
                            }
                            _ => {
                                return Err(LugliError::runtime(format!("Cannot set property '{}' on {}", name, object.type_name())));
                            }
                        }
                    } else {
                        return Err(LugliError::runtime("Property name must be a string"));
                    }
                }

                // Collection operations
                Instruction::MakeList(element_count) => {
                    let mut elements = Vec::new();
                    // Pop elements in reverse order (stack is LIFO)
                    for _ in 0..*element_count {
                        elements.insert(0, self.pop()?);
                    }
                    self.stack.push(Value::List(std::rc::Rc::new(std::cell::RefCell::new(elements))));
                }

                Instruction::MakeDict(pair_count) => {
                    let mut dict = HashMap::new();
                    // Pop key-value pairs in reverse order
                    for _ in 0..*pair_count {
                        let value = self.pop()?;
                        let key = self.pop()?;
                        if let Value::String(key_str) = key {
                            dict.insert(key_str, value);
                        } else {
                            return Err(LugliError::runtime("Dictionary keys must be strings"));
                        }
                    }
                    self.stack.push(Value::Dict(dict));
                }

                // Variable operations
                Instruction::LoadGlobal(name_index) => {
                    let var_name = &bytecode.constants[*name_index];

                    if let Value::String(name) = var_name {
                        if let Some(value) = self.globals.get(name) {
                            self.stack.push(value.clone());
                        } else {
                            return Err(LugliError::runtime(format!("Undefined variable: {}", name)));
                        }
                    } else {
                        return Err(LugliError::runtime("Variable name must be a string"));
                    }
                }

                Instruction::StoreGlobal(name_index) => {
                    let value = self.pop()?;
                    let var_name = &bytecode.constants[*name_index];

                    if let Value::String(name) = var_name {
                        self.globals.insert(name.clone(), value.clone());
                        self.stack.push(value); // Assignment expressions return the assigned value
                    } else {
                        return Err(LugliError::runtime("Variable name must be a string"));
                    }
                }

                // Function definition operations
                Instruction::DefineFunction(function_index) => {
                    let function_value = bytecode.constants[*function_index].clone();
                    self.stack.push(function_value);
                }

                Instruction::CallFunction(arg_count) => {
                    // For now, just redirect to the regular Call instruction
                    // Later this can be optimized for user-defined functions
                    let args: Vec<Value> = (0..*arg_count)
                        .map(|_| self.pop())
                        .collect::<Result<Vec<_>, _>>()?
                        .into_iter()
                        .rev()
                        .collect();

                    let function = self.pop()?;
                    match function {
                        Value::Function { arity, .. } => {
                            if args.len() != arity {
                                return Err(LugliError::runtime(format!("Function expects {} arguments, got {}", arity, args.len())));
                            }
                            return Err(LugliError::runtime("User-defined function calls not yet implemented"));
                        }
                        _ => return Err(LugliError::runtime(format!("Not callable: {}", function.type_name()))),
                    }
                }
            }
            self.ip += 1;
        }

        Ok(self.stack.pop().unwrap_or(Value::Null))
    }

    fn pop(&mut self) -> Result<Value, LugliError> {
        self.stack.pop().ok_or_else(|| LugliError::runtime("Stack underflow"))
    }

    fn ensure_local_exists(&mut self, index: usize) {
        while self.locals.len() <= index {
            self.locals.push(Value::Null);
        }
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new()
    }
}