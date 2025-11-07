use super::Compiler;
use crate::Instruction;
use lugli_ast::{Pattern, Stmt};
use lugli_common::{LugliError, Value};

impl Compiler {
    pub(super) fn compile_simple_stmt(&mut self, stmt: &Stmt) -> Result<bool, LugliError> {
        match stmt {
            Stmt::Expression {
                expr, ..
            } => {
                self.compile_expr(expr)?;
                // Keep expression result on stack for return value
                Ok(true) // Handled
            }
            Stmt::VarDecl {
                pattern,
                initializer,
                ..
            } => {
                // Compile initializer value
                if let Some(init) = initializer {
                    self.compile_expr(init)?;
                } else {
                    let null_index = self.add_constant(Value::Null);
                    self.emit_unknown(Instruction::Constant(null_index));
                }

                // Compile pattern binding
                self.compile_pattern_binding(pattern)?;

                Ok(true) // Handled
            }
            Stmt::DestructuringAssignment {
                pattern,
                value,
                ..
            } => {
                // Compile the value expression
                self.compile_expr(value)?;

                // Compile pattern assignment (assigns to existing variables)
                self.compile_pattern_assignment(pattern)?;

                Ok(true) // Handled
            }
            Stmt::Return {
                value, ..
            } => {
                if let Some(val) = value {
                    self.compile_expr(val)?;
                } else {
                    let null_index = self.add_constant(Value::Null);
                    self.emit_unknown(Instruction::Constant(null_index));
                }
                self.emit_unknown(Instruction::Return);
                Ok(true) // Handled
            }
            Stmt::StructDecl {
                data, ..
            } => {
                let name = &data.name;
                let fields = &data.fields;
                let methods = &data.methods;
                // Store struct metadata with field names and default values
                let struct_meta = Value::Dict(std::rc::Rc::new(std::cell::RefCell::new({
                    let mut pool = self.bytecode.string_pool.borrow_mut();
                    let mut meta = hashbrown::HashMap::new();

                    let type_key = pool.intern("__type__");
                    let type_value = pool.intern("struct");
                    meta.insert(type_key, Value::String(type_value));

                    let name_key = pool.intern("__name__");
                    let name_value = pool.intern(name);
                    meta.insert(name_key, Value::String(name_value));

                    // Store field names
                    let field_names: Vec<Value> = fields
                        .iter()
                        .map(|field| {
                            let id = pool.intern(&field.name);
                            Value::String(id)
                        })
                        .collect();
                    let fields_list = Value::List(std::rc::Rc::new(std::cell::RefCell::new(field_names)));
                    let fields_key = pool.intern("__fields__");
                    meta.insert(fields_key, fields_list);

                    // Store default values as a dict (field_name -> default_expr_as_constant)
                    let mut defaults_dict = hashbrown::HashMap::new();
                    for field in fields {
                        if let Some(expr) = &field.default {
                            // Evaluate constant default expressions at compile time
                            // Drop pool borrow before recursing
                            drop(pool);
                            if let Some(const_value) = self.try_evaluate_constant(expr) {
                                pool = self.bytecode.string_pool.borrow_mut();
                                let field_id = pool.intern(&field.name);
                                defaults_dict.insert(field_id, const_value);
                            } else {
                                pool = self.bytecode.string_pool.borrow_mut();
                            }
                        }
                    }
                    let defaults = Value::Dict(std::rc::Rc::new(std::cell::RefCell::new(defaults_dict)));
                    let defaults_key = pool.intern("__defaults__");
                    meta.insert(defaults_key, defaults);

                    drop(pool); // Release borrow before returning

                    meta
                })));

                let struct_index = self.add_constant(struct_meta);
                self.emit_unknown(Instruction::Constant(struct_index));

                let name_id = self.bytecode.string_pool.borrow_mut().intern(name);
                let name_index = self.add_constant(Value::String(name_id));
                self.emit_unknown(Instruction::StoreGlobal(name_index));

                // Compile methods as separate functions with TypeName_methodName convention
                for method in methods {
                    if let Stmt::FnDecl {
                        id,
                        name: method_name,
                        params,
                        return_type,
                        body,
                    } = method
                    {
                        // Create function with special name for struct methods
                        let static_method_name = format!("{}_{}", name, method_name);

                        // Compile as a regular function but with the special name
                        let modified_fn = Stmt::FnDecl {
                            id: *id,
                            name: static_method_name,
                            params: params.clone(),
                            return_type: return_type.clone(),
                            body: body.clone(),
                        };
                        self.compile_stmt(&modified_fn)?;
                    } else {
                        self.compile_stmt(method)?;
                    }
                }

                Ok(true) // Handled
            }
            Stmt::FnDecl {
                name,
                params,
                body,
                ..
            } => {
                let jump_over_body = self.emit_jump(Instruction::Jump(0));

                // Save compiler state
                let saved_locals = self.locals.clone();
                let saved_local_count = self.local_count;
                let saved_scope_depth = self.scope_depth;
                let saved_upvalues = self.upvalues.clone();
                let saved_upvalue_count = self.upvalue_count;

                // Extract param names (ignore type hints)
                let param_names: Vec<String> = params.iter().map(|(name, _)| name.clone()).collect();

                // Detect which variables will be captured from outer scope
                let captures = self.detect_captures(body, &saved_locals, &param_names);

                // Enter function scope
                self.locals.clear();
                self.local_count = 0;
                self.scope_depth += 1;

                // Set up upvalues for captured variables
                self.upvalues.clear();
                self.upvalue_count = 0;
                for capture_name in &captures {
                    self.upvalues.insert(capture_name.clone(), self.upvalue_count);
                    self.upvalue_count += 1;
                }

                // Declare parameters as local variables
                for param_name in &param_names {
                    self.declare_local(param_name.clone());
                }

                // Emit placeholder to reserve space for locals (will patch later)
                let param_count = param_names.len();
                let reserve_locals_ip = self.current_instruction();
                self.emit_unknown(Instruction::ReserveLocals(0)); // Placeholder

                // body_start must be set AFTER ReserveLocals so the VM executes it
                let body_start = reserve_locals_ip;

                // Compile function body
                for (i, stmt) in body.iter().enumerate() {
                    let is_last = i == body.len() - 1;
                    self.compile_stmt(stmt)?;

                    // Pop intermediate values from if-statements and expressions (except last statement)
                    if !is_last && matches!(stmt, Stmt::Expression { .. } | Stmt::If { .. }) {
                        self.emit_unknown(Instruction::Pop);
                    }
                }

                // Add implicit return if needed
                match body.last() {
                    Some(Stmt::Return {
                        ..
                    }) => {
                        // Explicit return already handled
                    }
                    Some(Stmt::Expression {
                        ..
                    })
                    | Some(Stmt::If {
                        ..
                    }) => {
                        // Expression result is on stack, just add Return instruction
                        self.emit_unknown(Instruction::Return);
                    }
                    _ => {
                        // No expression or other statement, return null
                        let null_index = self.add_constant(Value::Null);
                        self.emit_unknown(Instruction::Constant(null_index));
                        self.emit_unknown(Instruction::Return);
                    }
                }

                // Patch ReserveLocals with actual local count
                let total_local_count = self.local_count;
                let locals_to_reserve = total_local_count.saturating_sub(param_count);
                self.bytecode.instructions[reserve_locals_ip] = Instruction::ReserveLocals(locals_to_reserve);

                self.patch_jump(jump_over_body)?;

                // Restore compiler state
                self.locals = saved_locals.clone();
                self.local_count = saved_local_count;
                self.scope_depth = saved_scope_depth;
                self.upvalues = saved_upvalues;
                self.upvalue_count = saved_upvalue_count;

                // Create the function value (use param_names instead of params)
                let function_value = Value::Function {
                    name: name.clone(),
                    params: param_names,
                    body_start,
                    bytecode_id: 0, // Main bytecode
                };
                let function_index = self.add_constant(function_value);

                // If function has captures, emit MakeClosure and store as closure
                // Otherwise, use regular DefineFunction
                if captures.is_empty() {
                    // No captures - regular function
                    self.emit_unknown(Instruction::DefineFunction(function_index));
                } else {
                    // Has captures - create closure
                    let capture_indices: Vec<usize> = captures.iter().filter_map(|name| saved_locals.get(name).copied()).collect();

                    self.emit_unknown(Instruction::MakeClosure {
                        function_index,
                        capture_indices,
                    });
                }

                let name_id = self.bytecode.string_pool.borrow_mut().intern(name);
                let name_index = self.add_constant(Value::String(name_id));
                self.emit_unknown(Instruction::StoreGlobal(name_index));

                Ok(true) // Handled
            }
            Stmt::Import {
                module_path,
                items,
                alias,
                ..
            } => {
                // Convert module path to string (e.g., ["stdlib", "math"] -> "stdlib.math")
                let module_str = module_path.join(".");
                let module_id = self.bytecode.string_pool.borrow_mut().intern(&module_str);
                let module_idx = self.add_constant(Value::String(module_id));

                match items {
                    None => {
                        // import module [as alias]
                        // Determine binding name: alias if provided, else last component of path
                        let bind_name = if let Some(alias_name) = alias {
                            alias_name.clone()
                        } else {
                            module_path.last().ok_or_else(|| LugliError::runtime("Empty module path in import statement"))?.clone()
                        };

                        // ImportModule stores directly in globals with no stack effect
                        self.emit_unknown(Instruction::ImportModule {
                            module_idx,
                            bind_name,
                        });
                    }
                    Some(item_names) => {
                        // from module import item1, item2
                        // ImportFrom stores directly in globals with no stack effect
                        self.emit_unknown(Instruction::ImportFrom {
                            module_idx,
                            names: item_names.clone(),
                        });
                    }
                }

                Ok(true) // Handled
            }
            Stmt::Export {
                item, ..
            } => {
                // Export just compiles the inner statement
                // Module system will handle export metadata later
                self.compile_simple_stmt(item)?;
                Ok(true) // Handled
            }
            _ => Ok(false), // Not handled - control flow statements go to control_flow.rs
        }
    }

    pub(super) fn compile_pattern_binding(&mut self, pattern: &Pattern) -> Result<(), LugliError> {
        match pattern {
            Pattern::Identifier(name) => {
                // Original single-variable logic
                if self.scope_depth == 0 {
                    // Global scope
                    let name_id = self.bytecode.string_pool.borrow_mut().intern(name);
                    let name_index = self.add_constant(Value::String(name_id));
                    self.emit_unknown(Instruction::StoreGlobal(name_index));
                    // StoreGlobal uses peek() and leaves value on stack, need to pop it
                    self.emit_unknown(Instruction::Pop);
                } else {
                    // Local scope
                    let local_index = self.declare_local(name.clone());
                    self.emit_unknown(Instruction::Store(local_index));
                    // Store uses pop() and consumes the value, no extra pop needed
                }
            }

            Pattern::List(patterns) => {
                // Value is on stack - need to unpack it
                for (i, sub_pattern) in patterns.iter().enumerate() {
                    // Duplicate list on stack
                    self.emit_unknown(Instruction::Dup);

                    // Push index
                    let i_index = self.add_constant(Value::Number(i as f64));
                    self.emit_unknown(Instruction::Constant(i_index));

                    // Get element at index
                    self.emit_unknown(Instruction::GetIndex);

                    // Recursively bind sub-pattern
                    self.compile_pattern_binding(sub_pattern)?;
                }

                // Pop original list
                self.emit_unknown(Instruction::Pop);
            }

            Pattern::Dict(fields) => {
                // Value is on stack - need to unpack it
                for (key, sub_pattern) in fields {
                    // Duplicate dict on stack
                    self.emit_unknown(Instruction::Dup);

                    // Get property using key name
                    let key_id = self.bytecode.string_pool.borrow_mut().intern(key);
                    let key_index = self.add_constant(Value::String(key_id));
                    self.emit_unknown(Instruction::GetProperty(key_index));

                    // Recursively bind sub-pattern
                    self.compile_pattern_binding(sub_pattern)?;
                }

                // Pop original dict
                self.emit_unknown(Instruction::Pop);
            }

            Pattern::Wildcard => {
                // Discard value
                self.emit_unknown(Instruction::Pop);
            }

            Pattern::Literal(_) => {
                // Literals in patterns are only used for match arms, not variable declarations
                // For destructuring, we just pop the value
                self.emit_unknown(Instruction::Pop);
            }
        }
        Ok(())
    }

    pub(super) fn compile_pattern_assignment(&mut self, pattern: &Pattern) -> Result<(), LugliError> {
        match pattern {
            Pattern::Identifier(name) => {
                // Assign to existing variable
                if let Some(&local_index) = self.locals.get(name) {
                    // Local variable
                    self.emit_unknown(Instruction::Store(local_index));
                } else if let Some(&upvalue_index) = self.upvalues.get(name) {
                    // Upvalue (captured variable)
                    self.emit_unknown(Instruction::StoreUpvalue(upvalue_index));
                } else {
                    // Global variable
                    let name_id = self.bytecode.string_pool.borrow_mut().intern(name);
                    let name_index = self.add_constant(Value::String(name_id));
                    self.emit_unknown(Instruction::StoreGlobal(name_index));
                    self.emit_unknown(Instruction::Pop);
                }
            }

            Pattern::List(patterns) => {
                // Value is on stack - need to unpack it
                for (i, sub_pattern) in patterns.iter().enumerate() {
                    // Duplicate list on stack
                    self.emit_unknown(Instruction::Dup);

                    // Push index
                    let i_index = self.add_constant(Value::Number(i as f64));
                    self.emit_unknown(Instruction::Constant(i_index));

                    // Get element at index
                    self.emit_unknown(Instruction::GetIndex);

                    // Recursively assign sub-pattern
                    self.compile_pattern_assignment(sub_pattern)?;
                }

                // Pop original list
                self.emit_unknown(Instruction::Pop);
            }

            Pattern::Dict(fields) => {
                // Value is on stack - need to unpack it
                for (key, sub_pattern) in fields {
                    // Duplicate dict on stack
                    self.emit_unknown(Instruction::Dup);

                    // Get property using key name
                    let key_id = self.bytecode.string_pool.borrow_mut().intern(key);
                    let key_index = self.add_constant(Value::String(key_id));
                    self.emit_unknown(Instruction::GetProperty(key_index));

                    // Recursively assign sub-pattern
                    self.compile_pattern_assignment(sub_pattern)?;
                }

                // Pop original dict
                self.emit_unknown(Instruction::Pop);
            }

            Pattern::Wildcard => {
                // Discard value
                self.emit_unknown(Instruction::Pop);
            }

            Pattern::Literal(_) => {
                // Literals not valid in assignment patterns
                self.emit_unknown(Instruction::Pop);
            }
        }
        Ok(())
    }
}
