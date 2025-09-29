//! Visitor pattern implementation for AST traversal.

use crate::{Expr, Stmt, Program};

/// Trait for visiting AST nodes immutably.
pub trait Visitor<T = ()> where T: Default {
    /// Visit a program node.
    fn visit_program(&mut self, program: &Program) -> T {
        self.walk_program(program)
    }

    /// Visit a statement node.
    fn visit_stmt(&mut self, stmt: &Stmt) -> T {
        self.walk_stmt(stmt)
    }

    /// Visit an expression node.
    fn visit_expr(&mut self, expr: &Expr) -> T {
        self.walk_expr(expr)
    }

    /// Default program traversal.
    fn walk_program(&mut self, program: &Program) -> T {
        for stmt in &program.statements {
            self.visit_stmt(stmt);
        }
        Default::default()
    }

    /// Default statement traversal.
    fn walk_stmt(&mut self, stmt: &Stmt) -> T {
        match stmt {
            Stmt::Expression { expr, .. } => {
                self.visit_expr(expr);
            }
            Stmt::VarDecl { initializer, .. } => {
                if let Some(init) = initializer {
                    self.visit_expr(init);
                }
            }
            Stmt::FnDecl { body, .. } => {
                for stmt in body {
                    self.visit_stmt(stmt);
                }
            }
            Stmt::StructDecl { methods, .. } => {
                for method in methods {
                    self.visit_stmt(method);
                }
            }
            Stmt::If { condition, then_branch, elif_branches, else_branch, .. } => {
                self.visit_expr(condition);
                for stmt in then_branch {
                    self.visit_stmt(stmt);
                }
                for (cond, body) in elif_branches {
                    self.visit_expr(cond);
                    for stmt in body {
                        self.visit_stmt(stmt);
                    }
                }
                if let Some(else_body) = else_branch {
                    for stmt in else_body {
                        self.visit_stmt(stmt);
                    }
                }
            }
            Stmt::While { condition, body, .. } | Stmt::For { iterable: condition, body, .. } => {
                self.visit_expr(condition);
                for stmt in body {
                    self.visit_stmt(stmt);
                }
            }
            Stmt::Loop { body, .. } | Stmt::Block { statements: body, .. } => {
                for stmt in body {
                    self.visit_stmt(stmt);
                }
            }
            Stmt::Return { value, .. } => {
                if let Some(val) = value {
                    self.visit_expr(val);
                }
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => {
                // No children to visit
            }
        }
        Default::default()
    }

    /// Default expression traversal.
    fn walk_expr(&mut self, expr: &Expr) -> T {
        match expr {
            Expr::Binary { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            Expr::Unary { operand, .. } => {
                self.visit_expr(operand);
            }
            Expr::Call { callee, arguments, .. } => {
                self.visit_expr(callee);
                for arg in arguments {
                    self.visit_expr(arg);
                }
            }
            Expr::Get { object, .. } => {
                self.visit_expr(object);
            }
            Expr::Set { object, value, .. } => {
                self.visit_expr(object);
                self.visit_expr(value);
            }
            Expr::List { elements, .. } => {
                for elem in elements {
                    self.visit_expr(elem);
                }
            }
            Expr::Dict { pairs, .. } => {
                for (key, value) in pairs {
                    self.visit_expr(key);
                    self.visit_expr(value);
                }
            }
            Expr::Index { object, index, .. } => {
                self.visit_expr(object);
                self.visit_expr(index);
            }
            Expr::Literal { .. } | Expr::Identifier { .. } => {
                // No children to visit
            }
        }
        Default::default()
    }
}

/// Trait for visiting AST nodes mutably.
pub trait VisitorMut<T = ()> where T: Default {
    /// Visit a program node mutably.
    fn visit_program_mut(&mut self, program: &mut Program) -> T {
        self.walk_program_mut(program)
    }

    /// Visit a statement node mutably.
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) -> T {
        self.walk_stmt_mut(stmt)
    }

    /// Visit an expression node mutably.
    fn visit_expr_mut(&mut self, expr: &mut Expr) -> T {
        self.walk_expr_mut(expr)
    }

    /// Default mutable program traversal.
    fn walk_program_mut(&mut self, program: &mut Program) -> T {
        for stmt in &mut program.statements {
            self.visit_stmt_mut(stmt);
        }
        Default::default()
    }

    /// Default mutable statement traversal.
    fn walk_stmt_mut(&mut self, stmt: &mut Stmt) -> T {
        match stmt {
            Stmt::Expression { expr, .. } => {
                self.visit_expr_mut(expr);
            }
            Stmt::VarDecl { initializer, .. } => {
                if let Some(init) = initializer {
                    self.visit_expr_mut(init);
                }
            }
            Stmt::FnDecl { body, .. } => {
                for stmt in body {
                    self.visit_stmt_mut(stmt);
                }
            }
            Stmt::StructDecl { methods, .. } => {
                for method in methods {
                    self.visit_stmt_mut(method);
                }
            }
            Stmt::If { condition, then_branch, elif_branches, else_branch, .. } => {
                self.visit_expr_mut(condition);
                for stmt in then_branch {
                    self.visit_stmt_mut(stmt);
                }
                for (cond, body) in elif_branches {
                    self.visit_expr_mut(cond);
                    for stmt in body {
                        self.visit_stmt_mut(stmt);
                    }
                }
                if let Some(else_body) = else_branch {
                    for stmt in else_body {
                        self.visit_stmt_mut(stmt);
                    }
                }
            }
            Stmt::While { condition, body, .. } | Stmt::For { iterable: condition, body, .. } => {
                self.visit_expr_mut(condition);
                for stmt in body {
                    self.visit_stmt_mut(stmt);
                }
            }
            Stmt::Loop { body, .. } | Stmt::Block { statements: body, .. } => {
                for stmt in body {
                    self.visit_stmt_mut(stmt);
                }
            }
            Stmt::Return { value, .. } => {
                if let Some(val) = value {
                    self.visit_expr_mut(val);
                }
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => {
                // No children to visit
            }
        }
        Default::default()
    }

    /// Default mutable expression traversal.
    fn walk_expr_mut(&mut self, expr: &mut Expr) -> T {
        match expr {
            Expr::Binary { left, right, .. } => {
                self.visit_expr_mut(left);
                self.visit_expr_mut(right);
            }
            Expr::Unary { operand, .. } => {
                self.visit_expr_mut(operand);
            }
            Expr::Call { callee, arguments, .. } => {
                self.visit_expr_mut(callee);
                for arg in arguments {
                    self.visit_expr_mut(arg);
                }
            }
            Expr::Get { object, .. } => {
                self.visit_expr_mut(object);
            }
            Expr::Set { object, value, .. } => {
                self.visit_expr_mut(object);
                self.visit_expr_mut(value);
            }
            Expr::List { elements, .. } => {
                for elem in elements {
                    self.visit_expr_mut(elem);
                }
            }
            Expr::Dict { pairs, .. } => {
                for (key, value) in pairs {
                    self.visit_expr_mut(key);
                    self.visit_expr_mut(value);
                }
            }
            Expr::Index { object, index, .. } => {
                self.visit_expr_mut(object);
                self.visit_expr_mut(index);
            }
            Expr::Literal { .. } | Expr::Identifier { .. } => {
                // No children to visit
            }
        }
        Default::default()
    }
}