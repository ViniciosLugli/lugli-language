use super::Compiler;
use crate::Instruction;
use lugli_ast::Stmt;
use lugli_common::LugliError;

impl Compiler {
    pub(super) fn compile_control_flow_stmt(&mut self, stmt: &Stmt) -> Result<bool, LugliError> {
        match stmt {
            Stmt::If {
                data, ..
            } => {
                // Compile condition
                self.compile_expr(&data.condition)?;

                // Jump to else/end if condition is false
                let jump_to_else = self.emit_jump(Instruction::JumpIfFalse(0));

                // Compile then branch - leave last expression value on stack
                self.compile_branch_as_expr(&data.then_branch)?;

                let mut end_jumps = Vec::new();

                // Jump to end after then branch
                let jump_to_end = self.emit_jump(Instruction::Jump(0));
                end_jumps.push(jump_to_end);

                // Patch the else jump to point here (start of elif/else)
                self.patch_jump(jump_to_else)?;

                // Compile elif branches
                for (elif_condition, elif_body) in &data.elif_branches {
                    self.compile_expr(elif_condition)?;
                    let elif_jump = self.emit_jump(Instruction::JumpIfFalse(0));

                    self.compile_branch_as_expr(elif_body)?;

                    let end_jump = self.emit_jump(Instruction::Jump(0));
                    end_jumps.push(end_jump);

                    // Patch elif jump to point to next elif or else
                    self.patch_jump(elif_jump)?;
                }

                // Compile else branch if it exists
                if let Some(else_body) = &data.else_branch {
                    self.compile_branch_as_expr(else_body)?;
                } else {
                    // No else branch - push null to ensure consistent stack behavior
                    self.emit_unknown(crate::Instruction::LoadNull);
                }

                // Patch all end jumps to point here
                for jump in end_jumps {
                    self.patch_jump(jump)?;
                }

                Ok(true) // Handled
            }
            Stmt::While {
                condition,
                body,
                ..
            } => {
                let loop_start = self.current_instruction();

                // Push loop info onto stack for break/continue
                // For while loops, continue should re-evaluate condition
                self.loop_stack.push((loop_start, loop_start, Vec::new(), Vec::new()));

                // Compile condition
                self.compile_expr(condition)?;

                // Jump to end if condition is false
                let exit_jump = self.emit_jump(Instruction::JumpIfFalse(0));

                // Compile body
                for stmt in body {
                    self.compile_stmt(stmt)?;
                    // Pop expression results to avoid stack buildup in loops
                    if matches!(stmt, Stmt::Expression { .. }) {
                        self.emit_unknown(Instruction::Pop);
                    }
                }

                // Jump back to loop start
                self.emit_unknown(Instruction::Loop(loop_start));

                // Patch exit jump
                self.patch_jump(exit_jump)?;

                // Pop loop info and patch continue/break jumps
                if let Some((_, _, continue_jumps, break_jumps)) = self.loop_stack.pop() {
                    // For while loops, continue jumps to loop_start (condition check)
                    for jump in continue_jumps {
                        self.bytecode.instructions[jump] = Instruction::Jump(loop_start);
                    }
                    for jump in break_jumps {
                        self.patch_jump(jump)?;
                    }
                }

                Ok(true) // Handled
            }
            Stmt::Loop {
                body, ..
            } => {
                let loop_start = self.current_instruction();

                // Push loop info onto stack for break/continue
                // For infinite loops, continue should jump to loop top
                self.loop_stack.push((loop_start, loop_start, Vec::new(), Vec::new()));

                // Compile body
                for stmt in body {
                    self.compile_stmt(stmt)?;
                    // Pop expression results to avoid stack buildup in loops
                    if matches!(stmt, Stmt::Expression { .. }) {
                        self.emit_unknown(Instruction::Pop);
                    }
                }

                // Jump back to loop start (infinite loop)
                self.emit_unknown(Instruction::Loop(loop_start));

                // Pop loop info and patch continue/break jumps
                if let Some((_, _, continue_jumps, break_jumps)) = self.loop_stack.pop() {
                    // For infinite loops, continue jumps to loop_start (top of loop)
                    for jump in continue_jumps {
                        self.bytecode.instructions[jump] = Instruction::Jump(loop_start);
                    }
                    for jump in break_jumps {
                        self.patch_jump(jump)?;
                    }
                }

                Ok(true) // Handled
            }
            Stmt::For {
                pattern,
                iterable,
                body,
                ..
            } => {
                // Use unique names for nested loops
                let depth = self.loop_depth;
                self.loop_depth += 1;

                // Compile the iterable expression (should result in a list/range)
                self.compile_expr(iterable)?;

                // Store the iterable
                let iterable_local = self.declare_local(format!("__iterable_{}", depth));
                self.emit_unknown(Instruction::Store(iterable_local));

                // Initialize index to 0
                let index_local = self.declare_local(format!("__index_{}", depth));
                let zero_constant = self.add_constant(lugli_common::Value::Number(0.0));
                self.emit_unknown(Instruction::Constant(zero_constant));
                self.emit_unknown(Instruction::Store(index_local));

                let loop_start = self.current_instruction();

                // Push loop info for break/continue
                self.loop_stack.push((loop_start, 0, Vec::new(), Vec::new()));

                // Check if index < len(iterable)
                self.emit_unknown(Instruction::Load(index_local));
                self.emit_unknown(Instruction::Load(iterable_local));
                let len_id = self.bytecode.string_pool.borrow_mut().intern("len");
                let len_name = self.add_constant(lugli_common::Value::String(len_id));
                self.emit_unknown(Instruction::CallMethod(len_name, 0));

                // Now stack has: [index, length]
                // Check if index < length
                self.emit_unknown(Instruction::Less);
                let exit_jump = self.emit_jump(Instruction::JumpIfFalse(0));

                // Get element at current index
                self.emit_unknown(Instruction::Load(iterable_local));
                self.emit_unknown(Instruction::Load(index_local));
                self.emit_unknown(Instruction::GetIndex);

                // Bind pattern with element value on stack
                self.compile_pattern_binding(pattern)?;

                // Compile body
                for stmt in body {
                    self.compile_stmt(stmt)?;
                    // Pop expression results to avoid stack buildup in loops
                    if matches!(stmt, Stmt::Expression { .. }) {
                        self.emit_unknown(Instruction::Pop);
                    }
                }

                // Patch continue jumps to point to increment section
                let continue_target = self.current_instruction();
                if let Some((start, _, continue_jumps, breaks)) = self.loop_stack.pop() {
                    // Patch all continue jumps to jump to increment
                    for jump in continue_jumps {
                        self.bytecode.instructions[jump] = Instruction::Jump(continue_target);
                    }
                    self.loop_stack.push((start, continue_target, Vec::new(), breaks));
                }

                // Increment index
                self.emit_unknown(Instruction::Load(index_local));
                let one_constant = self.add_constant(lugli_common::Value::Number(1.0));
                self.emit_unknown(Instruction::Constant(one_constant));
                self.emit_unknown(Instruction::Add);
                self.emit_unknown(Instruction::Store(index_local));

                // Jump back to loop start
                self.emit_unknown(Instruction::Loop(loop_start));

                // Patch exit jump
                self.patch_jump(exit_jump)?;

                // Pop loop info and patch break jumps
                if let Some((_, _, _, break_jumps)) = self.loop_stack.pop() {
                    for jump in break_jumps {
                        self.patch_jump(jump)?;
                    }
                }

                // Restore loop depth
                self.loop_depth -= 1;

                Ok(true) // Handled
            }
            Stmt::Block {
                statements, ..
            } => {
                // Compile all statements in the block
                for stmt in statements {
                    self.compile_stmt(stmt)?;
                }
                Ok(true) // Handled
            }
            Stmt::Break {
                ..
            } => {
                if !self.loop_stack.is_empty() {
                    // Add this jump to the list of breaks to patch later
                    let break_jump = self.emit_jump(Instruction::Jump(0));
                    if let Some((_, _, _, break_jumps)) = self.loop_stack.last_mut() {
                        break_jumps.push(break_jump);
                    }
                    Ok(true)
                } else {
                    Err(LugliError::runtime("break statement outside of loop"))
                }
            }
            Stmt::Continue {
                ..
            } => {
                if !self.loop_stack.is_empty() {
                    // Emit placeholder jump - will be patched to correct target
                    let continue_jump = self.emit_jump(Instruction::Jump(0));
                    if let Some((_, _, continue_jumps, _)) = self.loop_stack.last_mut() {
                        continue_jumps.push(continue_jump);
                    }
                    Ok(true)
                } else {
                    Err(LugliError::runtime("continue statement outside of loop"))
                }
            }
            _ => Ok(false), // Not handled
        }
    }
}
