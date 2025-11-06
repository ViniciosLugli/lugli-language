use super::Compiler;
use crate::Instruction;
use lugli_ast::{Expr, FStringPart, LiteralValue};
use lugli_common::{LugliError, Value};

impl Compiler {
    pub(super) fn compile_expr(&mut self, expr: &Expr) -> Result<(), LugliError> {
        match expr {
            Expr::Literal {
                value, ..
            } => {
                match value {
                    LiteralValue::Number(n) => {
                        // Optimize small integers to use immediate instructions
                        if n.fract() == 0.0 && *n >= -128.0 && *n <= 127.0 {
                            self.emit_unknown(Instruction::LoadSmallInt(*n as i8));
                        } else if n.fract() == 0.0 && *n >= -32768.0 && *n <= 32767.0 {
                            self.emit_unknown(Instruction::LoadInt(*n as i16));
                        } else {
                            let constant_index = self.add_constant(Value::Number(*n));
                            self.emit_unknown(Instruction::Constant(constant_index));
                        }
                    }
                    LiteralValue::Boolean(true) => {
                        self.emit_unknown(Instruction::LoadTrue);
                    }
                    LiteralValue::Boolean(false) => {
                        self.emit_unknown(Instruction::LoadFalse);
                    }
                    LiteralValue::Null => {
                        self.emit_unknown(Instruction::LoadNull);
                    }
                    LiteralValue::String(s) => {
                        let id = self.bytecode.string_pool.borrow_mut().intern(s);
                        let constant_index = self.add_constant(Value::String(id));
                        self.emit_unknown(Instruction::Constant(constant_index));
                    }
                }
                Ok(())
            }
            Expr::Identifier {
                name, ..
            } => {
                // Check upvalues first (for closures)
                if let Some(&upvalue_index) = self.upvalues.get(name) {
                    self.emit_unknown(Instruction::LoadUpvalue(upvalue_index));
                    Ok(())
                } else if let Some(&local_index) = self.locals.get(name) {
                    self.emit_unknown(Instruction::Load(local_index));
                    Ok(())
                } else {
                    let name_id = self.bytecode.string_pool.borrow_mut().intern(name);
                    let name_index = self.add_constant(Value::String(name_id));
                    self.emit_unknown(Instruction::LoadGlobal(name_index));
                    Ok(())
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => {
                // Try to optimize arithmetic with small constant on the right
                let optimized = match operator {
                    lugli_lexer::TokenKind::Plus | lugli_lexer::TokenKind::Minus | lugli_lexer::TokenKind::Star => {
                        if let Expr::Literal {
                            value: LiteralValue::Number(n), ..
                        } = right.as_ref()
                        {
                            if n.fract() == 0.0 && *n >= -128.0 && *n <= 127.0 {
                                self.compile_expr(left)?;
                                match operator {
                                    lugli_lexer::TokenKind::Plus => self.emit_unknown(Instruction::AddInt(*n as i8)),
                                    lugli_lexer::TokenKind::Minus => self.emit_unknown(Instruction::SubInt(*n as i8)),
                                    lugli_lexer::TokenKind::Star => self.emit_unknown(Instruction::MulInt(*n as i8)),
                                    _ => unreachable!(),
                                }
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                    _ => false,
                };

                if !optimized {
                    // Fall back to general case
                    self.compile_expr(left)?;
                    self.compile_expr(right)?;

                    match operator {
                        lugli_lexer::TokenKind::Plus => self.emit_unknown(Instruction::Add),
                        lugli_lexer::TokenKind::Minus => self.emit_unknown(Instruction::Subtract),
                        lugli_lexer::TokenKind::Star => self.emit_unknown(Instruction::Multiply),
                        lugli_lexer::TokenKind::Slash => self.emit_unknown(Instruction::Divide),
                        lugli_lexer::TokenKind::IntegerDivision => self.emit_unknown(Instruction::IntegerDivide),
                        lugli_lexer::TokenKind::Percent => self.emit_unknown(Instruction::Modulo),
                        lugli_lexer::TokenKind::Power => self.emit_unknown(Instruction::Power),
                        lugli_lexer::TokenKind::EqualEqual => self.emit_unknown(Instruction::Equal),
                        lugli_lexer::TokenKind::BangEqual => self.emit_unknown(Instruction::NotEqual),
                        lugli_lexer::TokenKind::Greater => self.emit_unknown(Instruction::Greater),
                        lugli_lexer::TokenKind::GreaterEqual => self.emit_unknown(Instruction::GreaterEqual),
                        lugli_lexer::TokenKind::Less => self.emit_unknown(Instruction::Less),
                        lugli_lexer::TokenKind::LessEqual => self.emit_unknown(Instruction::LessEqual),
                        lugli_lexer::TokenKind::And => self.emit_unknown(Instruction::And),
                        lugli_lexer::TokenKind::Or => self.emit_unknown(Instruction::Or),
                        _ => return Err(LugliError::runtime(format!("Unsupported binary operator: {:?}", operator))),
                    }
                }
                Ok(())
            }
            Expr::Unary {
                operator,
                operand,
                ..
            } => {
                self.compile_expr(operand)?;

                match operator {
                    lugli_lexer::TokenKind::Minus => self.emit_unknown(Instruction::Negate),
                    lugli_lexer::TokenKind::Bang => self.emit_unknown(Instruction::Not),
                    _ => return Err(LugliError::runtime(format!("Unsupported unary operator: {:?}", operator))),
                }
                Ok(())
            }
            Expr::Call {
                data, ..
            } => {
                let callee = &data.callee;
                let arguments = &data.arguments;

                // Check if this is a method call (object.method())
                if let Expr::Get {
                    object,
                    name,
                    ..
                } = callee
                {
                    // This is a method call
                    // First compile the object
                    self.compile_expr(object)?;

                    // Then compile arguments (they'll be on stack in order)
                    for arg in arguments {
                        self.compile_expr(arg)?;
                    }

                    // Validate argument count before casting
                    if arguments.len() > 255 {
                        return Err(LugliError::runtime("Methods cannot have more than 255 arguments"));
                    }

                    // Add method name as constant and emit CallMethod instruction
                    let name_id = self.bytecode.string_pool.borrow_mut().intern(name);
                    let name_index = self.add_constant(Value::String(name_id));
                    self.emit_unknown(Instruction::CallMethod(name_index, arguments.len() as u8));
                } else {
                    // Regular function call
                    // Compile arguments first (they'll be on stack in order)
                    for arg in arguments {
                        self.compile_expr(arg)?;
                    }

                    // Validate argument count before casting
                    if arguments.len() > 255 {
                        return Err(LugliError::runtime("Functions cannot have more than 255 arguments"));
                    }

                    // Compile the function itself
                    self.compile_expr(callee)?;

                    // Emit call instruction with argument count
                    self.emit_unknown(Instruction::Call(arguments.len() as u8));
                }
                Ok(())
            }
            Expr::Get {
                object,
                name,
                ..
            } => {
                // Check if this is a global variable access (global.variable)
                if let Expr::Identifier {
                    name: obj_name, ..
                } = object.as_ref()
                    && obj_name == "global"
                {
                    // This is a variable access, not property access
                    let name_id = self.bytecode.string_pool.borrow_mut().intern(name);
                    let name_index = self.add_constant(Value::String(name_id));
                    self.emit_unknown(Instruction::LoadGlobal(name_index));
                    return Ok(());
                }

                // Regular property access
                // Compile the object expression
                self.compile_expr(object)?;

                // Add property name as constant and emit GetProperty instruction
                let name_id = self.bytecode.string_pool.borrow_mut().intern(name);
                let name_index = self.add_constant(Value::String(name_id));
                self.emit_unknown(Instruction::GetProperty(name_index));
                Ok(())
            }
            Expr::Set {
                object,
                name,
                value,
                ..
            } => {
                // Check if this is a variable assignment
                if let Expr::Identifier {
                    name: obj_name, ..
                } = object.as_ref()
                    && obj_name == "global"
                {
                    // This is a variable assignment - check local, then upvalue, then global
                    self.compile_expr(value)?;

                    // Check if it's a local variable first
                    if let Some(&local_index) = self.locals.get(name) {
                        self.emit_unknown(Instruction::Store(local_index));
                        self.emit_unknown(Instruction::Load(local_index));
                    } else if let Some(&upvalue_index) = self.upvalues.get(name) {
                        // It's an upvalue (captured from parent scope)
                        self.emit_unknown(Instruction::StoreUpvalue(upvalue_index));
                        self.emit_unknown(Instruction::LoadUpvalue(upvalue_index));
                    } else {
                        // It's a global variable
                        let name_id = self.bytecode.string_pool.borrow_mut().intern(name);
                        let name_index = self.add_constant(Value::String(name_id));
                        self.emit_unknown(Instruction::StoreGlobal(name_index));
                        self.emit_unknown(Instruction::LoadGlobal(name_index));
                    }
                    return Ok(());
                }

                if name == "[index]" {
                    if let Expr::List {
                        elements, ..
                    } = value.as_ref()
                    {
                        if elements.len() != 2 {
                            return Err(LugliError::runtime("Invalid index assignment structure"));
                        }

                        let index_expr = &elements[0];
                        let value_expr = &elements[1];

                        self.compile_expr(object)?;
                        self.compile_expr(index_expr)?;
                        self.compile_expr(value_expr)?;

                        self.emit_unknown(Instruction::SetIndex);
                        return Ok(());
                    } else {
                        return Err(LugliError::runtime("Invalid index assignment value"));
                    }
                }

                self.compile_expr(object)?;
                self.compile_expr(value)?;

                let name_id = self.bytecode.string_pool.borrow_mut().intern(name);
                let name_index = self.add_constant(Value::String(name_id));
                self.emit_unknown(Instruction::SetProperty(name_index));

                Ok(())
            }
            Expr::List {
                elements, ..
            } => {
                // Compile all elements (they'll be on stack in order)
                for element in elements {
                    self.compile_expr(element)?;
                }

                // Emit MakeList instruction with element count
                self.emit_unknown(Instruction::MakeList(elements.len()));
                Ok(())
            }
            Expr::Dict {
                pairs, ..
            } => {
                // Compile all key-value pairs (keys and values will be on stack alternating)
                for (key, value) in pairs {
                    self.compile_expr(key)?;
                    self.compile_expr(value)?;
                }

                // Emit MakeDict instruction with pair count
                self.emit_unknown(Instruction::MakeDict(pairs.len()));
                Ok(())
            }
            Expr::Index {
                object,
                index,
                ..
            } => {
                // Compile the object expression
                self.compile_expr(object)?;

                // Compile the index expression
                self.compile_expr(index)?;

                // Emit GetIndex instruction
                self.emit_unknown(Instruction::GetIndex);
                Ok(())
            }
            Expr::FString {
                parts, ..
            } => {
                // Desugar f-string into string concatenation
                // f"Hello {name}!" => "Hello " + str(name) + "!"

                if parts.is_empty() {
                    // Empty f-string => ""
                    let empty_id = self.bytecode.string_pool.borrow_mut().intern("");
                    let empty_str = self.add_constant(Value::String(empty_id));
                    self.emit_unknown(Instruction::Constant(empty_str));
                    return Ok(());
                }

                // Compile first part
                match &parts[0] {
                    FStringPart::Text(text) => {
                        let text_id = self.bytecode.string_pool.borrow_mut().intern(text);
                        let const_idx = self.add_constant(Value::String(text_id));
                        self.emit_unknown(Instruction::Constant(const_idx));
                    }
                    FStringPart::Expression(expr) => {
                        self.compile_expr(expr)?;
                        self.emit_unknown(Instruction::ToString);
                    }
                }

                // Concatenate remaining parts
                for part in &parts[1..] {
                    match part {
                        FStringPart::Text(text) => {
                            let text_id = self.bytecode.string_pool.borrow_mut().intern(text);
                            let const_idx = self.add_constant(Value::String(text_id));
                            self.emit_unknown(Instruction::Constant(const_idx));
                        }
                        FStringPart::Expression(expr) => {
                            self.compile_expr(expr)?;
                            self.emit_unknown(Instruction::ToString);
                        }
                    }
                    self.emit_unknown(Instruction::Add); // String concatenation uses Add
                }

                Ok(())
            }
            Expr::Function {
                params,
                body,
                ..
            } => {
                // Jump over function body (similar to FnDecl)
                let jump_over_body = self.emit_jump(Instruction::Jump(0));

                // Save compiler state
                let saved_locals = self.locals.clone();
                let saved_local_count = self.local_count;
                let saved_scope_depth = self.scope_depth;
                let saved_upvalues = self.upvalues.clone();
                let saved_upvalue_count = self.upvalue_count;

                // Detect which variables will be captured from outer scope
                let captures = self.detect_captures(body, &saved_locals, params);

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
                for param in params {
                    self.declare_local(param.clone());
                }

                // Emit placeholder to reserve space for locals (will patch later)
                let param_count = params.len();
                let reserve_locals_ip = self.current_instruction();
                self.emit_unknown(Instruction::ReserveLocals(0)); // Placeholder

                // body_start must be set AFTER ReserveLocals so the VM executes it
                let body_start = reserve_locals_ip;

                // Compile function body
                for stmt in body {
                    self.compile_stmt(stmt)?;
                }

                // Add implicit return if needed
                match body.last() {
                    Some(lugli_ast::Stmt::Return {
                        ..
                    }) => {
                        // Explicit return already handled
                    }
                    Some(lugli_ast::Stmt::Expression {
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
                let locals_to_reserve = if total_local_count > param_count {
                    total_local_count - param_count
                } else {
                    0
                };
                self.bytecode.instructions[reserve_locals_ip] = Instruction::ReserveLocals(locals_to_reserve);

                // Patch the jump to skip over the function body
                self.patch_jump(jump_over_body)?;

                // Restore compiler state
                self.locals = saved_locals.clone();
                self.local_count = saved_local_count;
                self.scope_depth = saved_scope_depth;
                self.upvalues = saved_upvalues;
                self.upvalue_count = saved_upvalue_count;

                // Create closure or function based on captures
                if captures.is_empty() {
                    // No captures - regular function
                    let function_value = Value::Function {
                        name: "<lambda>".to_string(),
                        params: params.clone(),
                        body_start,
                        bytecode_id: 0, // Main bytecode
                    };
                    let function_index = self.add_constant(function_value);
                    self.emit_unknown(Instruction::Constant(function_index));
                } else {
                    // Has captures - emit MakeClosure instruction
                    // First create the base function template
                    let function_value = Value::Function {
                        name: "<lambda>".to_string(),
                        params: params.clone(),
                        body_start,
                        bytecode_id: 0, // Main bytecode
                    };
                    let function_index = self.add_constant(function_value);

                    // Get the local indices of captured variables
                    let capture_indices: Vec<usize> = captures.iter().filter_map(|name| saved_locals.get(name).copied()).collect();

                    // Emit MakeClosure - VM will create closure with upvalues from stack
                    self.emit_unknown(Instruction::MakeClosure {
                        function_index,
                        capture_indices,
                    });
                }

                Ok(())
            }
            Expr::ListComprehension {
                data, ..
            } => {
                let element = &data.element;
                let clauses = &data.clauses;

                // Desugar nested comprehensions [expr for x in iter1 for y in iter2]
                // into nested loops with indexed iteration

                // Create empty result list
                self.emit_unknown(Instruction::MakeList(0));
                let depth = self.loop_depth;
                let result_local = self.declare_local(format!("__comp_result_{}", depth));
                self.emit_unknown(Instruction::Store(result_local));

                // Track loop structures for each clause
                struct LoopInfo {
                    index_local: usize,
                    loop_start: usize,
                    exit_jump: usize,
                    skip_jump: Option<usize>,
                }
                let mut loops: Vec<LoopInfo> = Vec::new();

                // Generate nested loops for each clause
                for (clause_idx, clause) in clauses.iter().enumerate() {
                    // Compile and store the iterable
                    self.compile_expr(&clause.iterable)?;
                    let iterable_local = self.declare_local(format!("__comp_iter_{}_{}", depth, clause_idx));
                    self.emit_unknown(Instruction::Store(iterable_local));

                    // Initialize index to 0
                    let index_local = self.declare_local(format!("__comp_idx_{}_{}", depth, clause_idx));
                    let zero_constant = self.add_constant(Value::Number(0.0));
                    self.emit_unknown(Instruction::Constant(zero_constant));
                    self.emit_unknown(Instruction::Store(index_local));

                    // Loop start
                    let loop_start = self.current_instruction();

                    // Check if index < len(iterable)
                    self.emit_unknown(Instruction::Load(index_local));
                    self.emit_unknown(Instruction::Load(iterable_local));
                    let len_id = self.bytecode.string_pool.borrow_mut().intern("len");
                    let len_name = self.add_constant(Value::String(len_id));
                    self.emit_unknown(Instruction::CallMethod(len_name, 0));
                    self.emit_unknown(Instruction::Less);
                    let exit_jump = self.emit_jump(Instruction::JumpIfFalse(0));

                    // Get element at current index and bind to pattern
                    self.emit_unknown(Instruction::Load(iterable_local));
                    self.emit_unknown(Instruction::Load(index_local));
                    self.emit_unknown(Instruction::GetIndex);

                    // Bind pattern with element value on stack
                    self.compile_pattern_binding(&clause.pattern)?;

                    // Optional condition check for this clause
                    let skip_jump = if let Some(cond) = &clause.condition {
                        self.compile_expr(cond)?;
                        Some(self.emit_jump(Instruction::JumpIfFalse(0)))
                    } else {
                        None
                    };

                    loops.push(LoopInfo {
                        index_local,
                        loop_start,
                        exit_jump,
                        skip_jump,
                    });

                    // If this is the last clause, append the element
                    if clause_idx == clauses.len() - 1 {
                        self.emit_unknown(Instruction::Load(result_local));
                        self.compile_expr(element)?;
                        let push_id = self.bytecode.string_pool.borrow_mut().intern("push");
                        let push_const = self.add_constant(Value::String(push_id));
                        self.emit_unknown(Instruction::CallMethod(push_const, 1));
                        self.emit_unknown(Instruction::Pop); // Pop return value from push
                    }
                }

                // Close all nested loops (in reverse order)
                for loop_info in loops.iter().rev() {
                    // Patch skip jump to point here (increment of this specific loop)
                    if let Some(skip) = loop_info.skip_jump {
                        self.patch_jump(skip)?;
                    }

                    // Increment index
                    self.emit_unknown(Instruction::Load(loop_info.index_local));
                    let one_constant = self.add_constant(Value::Number(1.0));
                    self.emit_unknown(Instruction::Constant(one_constant));
                    self.emit_unknown(Instruction::Add);
                    self.emit_unknown(Instruction::Store(loop_info.index_local));

                    // Jump back to this loop's start
                    self.emit_unknown(Instruction::Loop(loop_info.loop_start));

                    // Patch exit jump
                    self.patch_jump(loop_info.exit_jump)?;
                }

                // Load result list onto stack
                self.emit_unknown(Instruction::Load(result_local));

                // Clean up locals (compiler-generated temps only)
                // Pattern variables will be cleaned up by scope management
                self.locals.retain(|name, _| !name.starts_with("__comp_"));

                Ok(())
            }
            Expr::Match {
                value,
                arms,
                ..
            } => {
                // Match expression bytecode generation
                // See docs/PATTERN_MATCHING_BYTECODE.md for detailed explanation
                //
                // Stack discipline: match_value persists across all arms
                // Each arm tests pattern, optionally tests guard, then either:
                //   - Executes body and jumps to end (success)
                //   - Jumps to next arm (failure)
                //
                // Critical: JumpIfFalse CONSUMES the boolean value

                use lugli_ast::Pattern;

                self.compile_expr(value)?;

                let mut arm_end_jumps = Vec::new();

                for arm in arms.iter() {
                    let arm_local_start = self.local_count;
                    let mut arm_locals = Vec::new();

                    // Compile pattern test → Stack: [match_value, pattern_result]
                    match &arm.pattern {
                        Pattern::Literal(lit_val) => {
                            self.emit_unknown(Instruction::Dup);
                            let lit_const = self.add_constant(self.literal_to_value(lit_val));
                            self.emit_unknown(Instruction::Constant(lit_const));
                            self.emit_unknown(Instruction::Equal);
                        }
                        Pattern::Identifier(name) => {
                            // Bind match value to variable (local or global depending on scope)
                            self.emit_unknown(Instruction::Dup);
                            if self.scope_depth == 0 {
                                // Global scope
                                let name_id = self.bytecode.string_pool.borrow_mut().intern(name);
                                let name_index = self.add_constant(Value::String(name_id));
                                self.emit_unknown(Instruction::StoreGlobal(name_index));
                                // Note: StoreGlobal uses peek, doesn't pop, so we're left with the value on stack
                            } else {
                                // Local scope
                                let var_local = self.declare_local(name.clone());
                                arm_locals.push(name.clone());
                                self.emit_unknown(Instruction::Store(var_local)); // Store pops the dup
                            }
                            let true_const = self.add_constant(Value::Bool(true));
                            self.emit_unknown(Instruction::Constant(true_const));
                        }
                        Pattern::List(_) | Pattern::Dict(_) => {
                            // Complex patterns in match arms not yet supported
                            // Treat as wildcard for now
                            let true_const = self.add_constant(Value::Bool(true));
                            self.emit_unknown(Instruction::Constant(true_const));
                        }
                        Pattern::Wildcard => {
                            let true_const = self.add_constant(Value::Bool(true));
                            self.emit_unknown(Instruction::Constant(true_const));
                        }
                    }

                    let skip_to_next_arm = if let Some(guard) = &arm.guard {
                        // Pattern with guard: pattern && guard
                        let pattern_failed = self.emit_jump(Instruction::JumpIfFalse(0));
                        // JumpIfFalse consumed pattern_result → Stack: [match_value]

                        self.compile_expr(guard)?;
                        // Stack: [match_value, guard_result]

                        let guard_failed = self.emit_jump(Instruction::JumpIfFalse(0));
                        // JumpIfFalse consumed guard_result → Stack: [match_value]

                        // Both failure paths land here with Stack: [match_value]
                        self.patch_jump(pattern_failed)?;

                        Some(guard_failed)
                    } else {
                        // Pattern only (no guard)
                        let skip = self.emit_jump(Instruction::JumpIfFalse(0));
                        // JumpIfFalse consumed pattern_result → Stack: [match_value]
                        Some(skip)
                    };

                    // Success: pattern (and guard if present) matched
                    // Stack: [match_value]
                    self.emit_unknown(Instruction::Pop);
                    // Stack: []

                    self.compile_expr(&arm.body)?;
                    // Stack: [body_result]

                    let end_jump = self.emit_jump(Instruction::Jump(0));
                    arm_end_jumps.push(end_jump);

                    // Failure: jump target for failed pattern/guard
                    if let Some(skip_jump) = skip_to_next_arm {
                        self.patch_jump(skip_jump)?;
                    }
                    // Stack: [match_value] - JumpIfFalse already cleaned up boolean

                    // Cleanup: remove arm-local pattern bindings
                    for var_name in arm_locals {
                        self.locals.remove(&var_name);
                    }
                    self.local_count = arm_local_start;
                }

                // All success paths jump here
                for jump in arm_end_jumps {
                    self.patch_jump(jump)?;
                }
                // Stack: [body_result] from whichever arm matched

                Ok(())
            }
            Expr::Block {
                statements, ..
            } => {
                for (i, stmt) in statements.iter().enumerate() {
                    let is_last = i == statements.len() - 1;

                    if is_last {
                        match stmt {
                            lugli_ast::Stmt::Expression {
                                expr, ..
                            } => {
                                // Check if the expression is an assignment (Set)
                                // Assignments shouldn't be the return value of a block
                                if matches!(expr, lugli_ast::Expr::Set { .. }) {
                                    self.compile_expr(expr)?;
                                    self.emit_unknown(Instruction::Pop);
                                    self.emit_unknown(Instruction::LoadNull);
                                } else {
                                    self.compile_expr(expr)?;
                                }
                            }
                            lugli_ast::Stmt::If {
                                ..
                            } => {
                                // If-statements leave a value on the stack (from compile_branch_as_expr)
                                self.compile_stmt(stmt)?;
                            }
                            _ => {
                                self.compile_stmt(stmt)?;
                                self.emit_unknown(Instruction::LoadNull);
                            }
                        }
                    } else {
                        self.compile_stmt(stmt)?;
                        // Pop intermediate values from expression statements and if-statements
                        if matches!(stmt, lugli_ast::Stmt::Expression { .. } | lugli_ast::Stmt::If { .. }) {
                            self.emit_unknown(Instruction::Pop);
                        }
                    }
                }

                if statements.is_empty() {
                    self.emit_unknown(Instruction::LoadNull);
                }

                Ok(())
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.compile_expr(condition)?;

                // JumpIfFalse pops the condition automatically
                let else_jump = self.emit_jump(Instruction::JumpIfFalse(0));

                self.compile_expr(then_branch)?;

                let end_jump = self.emit_jump(Instruction::Jump(0));

                self.patch_jump(else_jump)?;

                if let Some(else_expr) = else_branch {
                    self.compile_expr(else_expr)?;
                } else {
                    self.emit_unknown(Instruction::LoadNull);
                }

                self.patch_jump(end_jump)?;

                Ok(())
            }
        }
    }

    pub(super) fn literal_to_value(&self, literal: &LiteralValue) -> Value {
        match literal {
            LiteralValue::Number(n) => Value::Number(*n),
            LiteralValue::String(s) => {
                let id = self.bytecode.string_pool.borrow_mut().intern(s);
                Value::String(id)
            }
            LiteralValue::Boolean(b) => Value::Bool(*b),
            LiteralValue::Null => Value::Null,
        }
    }

    pub(crate) fn detect_captures(
        &self,
        body: &[lugli_ast::Stmt],
        outer_locals: &hashbrown::HashMap<String, usize>,
        params: &[String],
    ) -> Vec<String> {
        use std::collections::HashSet;
        let mut captures = HashSet::new();

        for stmt in body {
            self.find_captured_identifiers(stmt, outer_locals, params, &mut captures);
        }

        captures.into_iter().collect()
    }

    fn find_captured_identifiers(
        &self,
        stmt: &lugli_ast::Stmt,
        outer_locals: &hashbrown::HashMap<String, usize>,
        params: &[String],
        captures: &mut std::collections::HashSet<String>,
    ) {
        use lugli_ast::Stmt;

        match stmt {
            Stmt::Expression {
                expr, ..
            } => {
                self.find_captured_identifiers_in_expr(expr, outer_locals, params, captures);
            }
            Stmt::Return {
                value: Some(expr), ..
            } => {
                self.find_captured_identifiers_in_expr(expr, outer_locals, params, captures);
            }
            Stmt::VarDecl {
                initializer: Some(expr), ..
            } => {
                self.find_captured_identifiers_in_expr(expr, outer_locals, params, captures);
                // Don't capture the variable being declared
            }
            Stmt::VarDecl {
                initializer: None, ..
            } => {
                // No initializer, nothing to capture
            }
            Stmt::If {
                data, ..
            } => {
                self.find_captured_identifiers_in_expr(&data.condition, outer_locals, params, captures);
                for s in &data.then_branch {
                    self.find_captured_identifiers(s, outer_locals, params, captures);
                }
                for (elif_cond, elif_body) in &data.elif_branches {
                    self.find_captured_identifiers_in_expr(elif_cond, outer_locals, params, captures);
                    for s in elif_body {
                        self.find_captured_identifiers(s, outer_locals, params, captures);
                    }
                }
                if let Some(else_body) = &data.else_branch {
                    for s in else_body {
                        self.find_captured_identifiers(s, outer_locals, params, captures);
                    }
                }
            }
            Stmt::While {
                condition,
                body,
                ..
            }
            | Stmt::For {
                iterable: condition,
                body,
                ..
            } => {
                self.find_captured_identifiers_in_expr(condition, outer_locals, params, captures);
                for s in body {
                    self.find_captured_identifiers(s, outer_locals, params, captures);
                }
            }
            Stmt::Loop {
                body, ..
            } => {
                for s in body {
                    self.find_captured_identifiers(s, outer_locals, params, captures);
                }
            }
            _ => {}
        }
    }

    fn find_captured_identifiers_in_expr(
        &self,
        expr: &Expr,
        outer_locals: &hashbrown::HashMap<String, usize>,
        params: &[String],
        captures: &mut std::collections::HashSet<String>,
    ) {
        match expr {
            Expr::Identifier {
                name, ..
            } => {
                // Check if this identifier exists in outer scope and is not a parameter of this function
                let is_param = params.contains(name);
                if outer_locals.contains_key(name) && !is_param {
                    captures.insert(name.clone());
                }
            }
            Expr::Binary {
                left,
                right,
                ..
            } => {
                self.find_captured_identifiers_in_expr(left, outer_locals, params, captures);
                self.find_captured_identifiers_in_expr(right, outer_locals, params, captures);
            }
            Expr::Unary {
                operand, ..
            } => {
                self.find_captured_identifiers_in_expr(operand, outer_locals, params, captures);
            }
            Expr::Call {
                data, ..
            } => {
                self.find_captured_identifiers_in_expr(&data.callee, outer_locals, params, captures);
                for arg in &data.arguments {
                    self.find_captured_identifiers_in_expr(arg, outer_locals, params, captures);
                }
            }
            Expr::Get {
                object, ..
            } => {
                self.find_captured_identifiers_in_expr(object, outer_locals, params, captures);
            }
            Expr::Set {
                object,
                value,
                ..
            } => {
                self.find_captured_identifiers_in_expr(object, outer_locals, params, captures);
                self.find_captured_identifiers_in_expr(value, outer_locals, params, captures);
            }
            Expr::List {
                elements, ..
            } => {
                for elem in elements {
                    self.find_captured_identifiers_in_expr(elem, outer_locals, params, captures);
                }
            }
            Expr::Dict {
                pairs, ..
            } => {
                for (key, value) in pairs {
                    self.find_captured_identifiers_in_expr(key, outer_locals, params, captures);
                    self.find_captured_identifiers_in_expr(value, outer_locals, params, captures);
                }
            }
            Expr::Index {
                object,
                index,
                ..
            } => {
                self.find_captured_identifiers_in_expr(object, outer_locals, params, captures);
                self.find_captured_identifiers_in_expr(index, outer_locals, params, captures);
            }
            Expr::FString {
                parts, ..
            } => {
                for part in parts.as_ref() {
                    if let FStringPart::Expression(expr) = part {
                        self.find_captured_identifiers_in_expr(expr.as_ref(), outer_locals, params, captures);
                    }
                }
            }
            Expr::Function {
                body, ..
            } => {
                // Don't recurse into nested functions - they have their own capture detection
                for stmt in body {
                    self.find_captured_identifiers(stmt, outer_locals, params, captures);
                }
            }
            Expr::ListComprehension {
                data, ..
            } => {
                self.find_captured_identifiers_in_expr(&data.element, outer_locals, params, captures);
                for clause in &data.clauses {
                    self.find_captured_identifiers_in_expr(&clause.iterable, outer_locals, params, captures);
                    if let Some(cond) = &clause.condition {
                        self.find_captured_identifiers_in_expr(cond, outer_locals, params, captures);
                    }
                }
            }
            Expr::Block {
                statements, ..
            } => {
                for stmt in statements {
                    self.find_captured_identifiers(stmt, outer_locals, params, captures);
                }
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.find_captured_identifiers_in_expr(condition, outer_locals, params, captures);
                self.find_captured_identifiers_in_expr(then_branch, outer_locals, params, captures);
                if let Some(else_expr) = else_branch {
                    self.find_captured_identifiers_in_expr(else_expr, outer_locals, params, captures);
                }
            }
            _ => {}
        }
    }
}
