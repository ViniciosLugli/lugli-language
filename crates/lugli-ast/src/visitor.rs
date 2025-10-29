use crate::{Expr, Program, Stmt};

pub trait Visitor<T = ()>
where
    T: Default,
{
    fn visit_program(&mut self, program: &Program) -> T {
        self.walk_program(program)
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> T {
        self.walk_stmt(stmt)
    }

    fn visit_expr(&mut self, expr: &Expr) -> T {
        self.walk_expr(expr)
    }

    fn walk_program(&mut self, program: &Program) -> T {
        for stmt in &program.statements {
            self.visit_stmt(stmt);
        }
        Default::default()
    }

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
            Stmt::StructDecl { data, .. } => {
                for (_, default_value) in &data.fields {
                    if let Some(expr) = default_value {
                        self.visit_expr(expr);
                    }
                }
                for method in &data.methods {
                    self.visit_stmt(method);
                }
            }
            Stmt::If { data, .. } => {
                self.visit_expr(&data.condition);
                for stmt in &data.then_branch {
                    self.visit_stmt(stmt);
                }
                for (cond, body) in &data.elif_branches {
                    self.visit_expr(cond);
                    for stmt in body {
                        self.visit_stmt(stmt);
                    }
                }
                if let Some(else_body) = &data.else_branch {
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
            Stmt::Break { .. } | Stmt::Continue { .. } => {}
            Stmt::Import { .. } => {}
            Stmt::Export { item, .. } => {
                self.visit_stmt(item);
            }
        }
        Default::default()
    }

    fn walk_expr(&mut self, expr: &Expr) -> T {
        match expr {
            Expr::Binary { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            Expr::Unary { operand, .. } => {
                self.visit_expr(operand);
            }
            Expr::Call { data, .. } => {
                self.visit_expr(&data.callee);
                for arg in &data.arguments {
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
            Expr::FString { parts, .. } => {
                use crate::FStringPart;
                for part in parts.as_ref() {
                    if let FStringPart::Expression(expr) = part {
                        self.visit_expr(expr);
                    }
                }
            }
            Expr::Function { body, .. } => {
                for stmt in body {
                    self.visit_stmt(stmt);
                }
            }
            Expr::ListComprehension { data, .. } => {
                self.visit_expr(&data.element);
                self.visit_expr(&data.iterable);
                if let Some(cond) = &data.condition {
                    self.visit_expr(cond);
                }
            }
            Expr::Match { value, arms, .. } => {
                self.visit_expr(value);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.visit_expr(guard);
                    }
                    self.visit_expr(&arm.body);
                }
            }
            Expr::Literal { .. } | Expr::Identifier { .. } => {}
        }
        Default::default()
    }
}

pub trait VisitorMut<T = ()>
where
    T: Default,
{
    fn visit_program_mut(&mut self, program: &mut Program) -> T {
        self.walk_program_mut(program)
    }

    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) -> T {
        self.walk_stmt_mut(stmt)
    }

    fn visit_expr_mut(&mut self, expr: &mut Expr) -> T {
        self.walk_expr_mut(expr)
    }

    fn walk_program_mut(&mut self, program: &mut Program) -> T {
        for stmt in &mut program.statements {
            self.visit_stmt_mut(stmt);
        }
        Default::default()
    }

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
            Stmt::StructDecl { data, .. } => {
                for (_, default_value) in &mut data.fields {
                    if let Some(expr) = default_value {
                        self.visit_expr_mut(expr);
                    }
                }
                for method in &mut data.methods {
                    self.visit_stmt_mut(method);
                }
            }
            Stmt::If { data, .. } => {
                self.visit_expr_mut(&mut data.condition);
                for stmt in &mut data.then_branch {
                    self.visit_stmt_mut(stmt);
                }
                for (cond, body) in &mut data.elif_branches {
                    self.visit_expr_mut(cond);
                    for stmt in body {
                        self.visit_stmt_mut(stmt);
                    }
                }
                if let Some(else_body) = &mut data.else_branch {
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
            Stmt::Break { .. } | Stmt::Continue { .. } => {}
            Stmt::Import { .. } => {}
            Stmt::Export { item, .. } => {
                self.visit_stmt_mut(item);
            }
        }
        Default::default()
    }

    fn walk_expr_mut(&mut self, expr: &mut Expr) -> T {
        match expr {
            Expr::Binary { left, right, .. } => {
                self.visit_expr_mut(left);
                self.visit_expr_mut(right);
            }
            Expr::Unary { operand, .. } => {
                self.visit_expr_mut(operand);
            }
            Expr::Call { data, .. } => {
                self.visit_expr_mut(&mut data.callee);
                for arg in &mut data.arguments {
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
            Expr::FString { parts, .. } => {
                use crate::FStringPart;
                for part in parts.as_mut() {
                    if let FStringPart::Expression(expr) = part {
                        self.visit_expr_mut(expr);
                    }
                }
            }
            Expr::Function { body, .. } => {
                for stmt in body {
                    self.visit_stmt_mut(stmt);
                }
            }
            Expr::ListComprehension { data, .. } => {
                self.visit_expr_mut(&mut data.element);
                self.visit_expr_mut(&mut data.iterable);
                if let Some(cond) = &mut data.condition {
                    self.visit_expr_mut(cond);
                }
            }
            Expr::Match { value, arms, .. } => {
                self.visit_expr_mut(value);
                for arm in arms {
                    if let Some(guard) = &mut arm.guard {
                        self.visit_expr_mut(guard);
                    }
                    self.visit_expr_mut(&mut arm.body);
                }
            }
            Expr::Literal { .. } | Expr::Identifier { .. } => {}
        }
        Default::default()
    }
}
