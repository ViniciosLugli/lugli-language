use lugli_common::LugliError;
use lugli_ast::Stmt;
use crate::Instruction;
use super::Compiler;

impl Compiler {
    pub(super) fn compile_control_flow_stmt(&mut self, stmt: &Stmt) -> Result<bool, LugliError> {
        match stmt {
            Stmt::If { condition, then_branch, elif_branches, else_branch, .. } => {
                // Compile condition
                self.compile_expr(condition)?;

                // Jump to else/end if condition is false
                let jump_to_else = self.emit_jump(Instruction::JumpIfFalse(0));

                // Compile then branch
                for stmt in then_branch {
                    self.compile_stmt(stmt)?;
                }

                let mut elif_jumps = Vec::new();
                let mut end_jumps = Vec::new();

                // Jump to end after then branch
                let jump_to_end = self.emit_jump(Instruction::Jump(0));
                end_jumps.push(jump_to_end);

                // Patch the else jump to point here (start of elif/else)
                self.patch_jump(jump_to_else);

                // Compile elif branches
                for (elif_condition, elif_body) in elif_branches {
                    self.compile_expr(elif_condition)?;
                    let elif_jump = self.emit_jump(Instruction::JumpIfFalse(0));

                    for stmt in elif_body {
                        self.compile_stmt(stmt)?;
                    }

                    let end_jump = self.emit_jump(Instruction::Jump(0));
                    end_jumps.push(end_jump);
                    elif_jumps.push(elif_jump);
                }

                // Patch elif jumps to point to next elif or else
                for jump in elif_jumps {
                    self.patch_jump(jump);
                }

                // Compile else branch if it exists
                if let Some(else_body) = else_branch {
                    for stmt in else_body {
                        self.compile_stmt(stmt)?;
                    }
                }

                // Patch all end jumps to point here
                for jump in end_jumps {
                    self.patch_jump(jump);
                }

                Ok(true) // Handled
            }
            Stmt::While { condition, body, .. } => {
                let loop_start = self.current_instruction();

                // Compile condition
                self.compile_expr(condition)?;

                // Jump to end if condition is false
                let exit_jump = self.emit_jump(Instruction::JumpIfFalse(0));

                // Compile body
                for stmt in body {
                    self.compile_stmt(stmt)?;
                }

                // Jump back to loop start
                self.emit(Instruction::Loop(loop_start));

                // Patch exit jump
                self.patch_jump(exit_jump);

                Ok(true) // Handled
            }
            _ => Ok(false) // Not handled
        }
    }

}