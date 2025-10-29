use super::Compiler;
use crate::Instruction;
use lugli_ast::{AstNode, Stmt};
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
                name,
                initializer,
                ..
            } => {
                // Declare local first if in local scope (to get correct index)
                let storage_info = if self.scope_depth == 0 {
                    None // Global scope
                } else {
                    Some(self.declare_local(name.clone())) // Local scope
                };

                // Compile initializer value
                if let Some(init) = initializer {
                    self.compile_expr(init)?;
                } else {
                    let null_index = self.add_constant(Value::Null);
                    self.emit(Instruction::Constant(null_index));
                }

                // Emit appropriate store instruction
                if let Some(local_index) = storage_info {
                    // Local scope - store as local variable
                    self.emit(Instruction::Store(local_index));
                } else {
                    // Global scope - store as global variable
                    let name_index = self.add_constant(Value::String(name.clone()));
                    self.emit(Instruction::StoreGlobal(name_index));
                }

                Ok(true) // Handled
            }
            Stmt::Return {
                value, ..
            } => {
                if let Some(val) = value {
                    self.compile_expr(val)?;
                } else {
                    let null_index = self.add_constant(Value::Null);
                    self.emit(Instruction::Constant(null_index));
                }
                self.emit(Instruction::Return);
                Ok(true) // Handled
            }
            Stmt::StructDecl {
                name,
                fields,
                methods,
                ..
            } => {
                // Store struct metadata with field names and default values
                let struct_meta = Value::Dict(std::rc::Rc::new(std::cell::RefCell::new({
                    let mut meta = hashbrown::HashMap::new();
                    meta.insert("__type__".to_string(), Value::String("struct".to_string()));
                    meta.insert("__name__".to_string(), Value::String(name.clone()));

                    // Store field names
                    let field_names: Vec<Value> = fields.iter().map(|(name, _)| Value::String(name.clone())).collect();
                    let fields_list = Value::List(std::rc::Rc::new(std::cell::RefCell::new(field_names)));
                    meta.insert("__fields__".to_string(), fields_list);

                    // Store default values as a dict (field_name -> default_expr_as_constant)
                    let mut defaults_dict = hashbrown::HashMap::new();
                    for (field_name, default_expr) in fields {
                        if let Some(expr) = default_expr {
                            // Evaluate constant default expressions at compile time
                            if let Some(const_value) = self.try_evaluate_constant(expr) {
                                defaults_dict.insert(field_name.clone(), const_value);
                            }
                        }
                    }
                    let defaults = Value::Dict(std::rc::Rc::new(std::cell::RefCell::new(defaults_dict)));
                    meta.insert("__defaults__".to_string(), defaults);

                    meta
                })));

                let struct_index = self.add_constant(struct_meta);
                self.emit(Instruction::Constant(struct_index));

                let name_index = self.add_constant(Value::String(name.clone()));
                self.emit(Instruction::StoreGlobal(name_index));

                // Compile methods as separate functions with TypeName_methodName convention
                for method in methods {
                    if let Stmt::FnDecl {
                        name: method_name,
                        params,
                        body,
                        ..
                    } = method
                    {
                        // Create function with special name for struct methods
                        let static_method_name = format!("{}_{}", name, method_name);

                        // Compile as a regular function but with the special name
                        let modified_fn = Stmt::FnDecl {
                            name: static_method_name,
                            params: params.clone(),
                            body: body.clone(),
                            span: *method.span(),
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

                let body_start = self.current_instruction();

                let saved_locals = self.locals.clone();
                let saved_local_count = self.local_count;
                let saved_scope_depth = self.scope_depth;

                self.locals.clear();
                self.local_count = 0;
                self.scope_depth += 1; // Enter function scope

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

                self.patch_jump(jump_over_body)?;

                self.locals = saved_locals;
                self.local_count = saved_local_count;
                self.scope_depth = saved_scope_depth; // Restore scope depth

                let function_value = Value::Function {
                    name: name.clone(),
                    params: params.clone(),
                    body_start,
                    bytecode_id: 0, // Main bytecode
                };

                let function_index = self.add_constant(function_value);
                self.emit(Instruction::DefineFunction(function_index));

                let name_index = self.add_constant(Value::String(name.clone()));
                self.emit(Instruction::StoreGlobal(name_index));

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
                let module_idx = self.add_constant(Value::String(module_str.clone()));

                match items {
                    None => {
                        // import module [as alias]
                        // Determine binding name: alias if provided, else last component of path
                        let bind_name = alias.as_ref().map(|s| s.clone()).unwrap_or_else(|| module_path.last().unwrap().clone());

                        // ImportModule stores directly in globals with no stack effect
                        self.emit(Instruction::ImportModule {
                            module_idx,
                            bind_name,
                        });
                    }
                    Some(item_names) => {
                        // from module import item1, item2
                        // ImportFrom stores directly in globals with no stack effect
                        self.emit(Instruction::ImportFrom {
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
}
