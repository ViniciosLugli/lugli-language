//! Statement AST nodes for Lugli.

use lugli_common::Span;
use crate::{AstNode, Expr};

/// Statements in Lugli.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Expression statements
    Expression {
        expr: Expr,
        span: Span,
    },

    /// Variable declarations (create x = 5)
    VarDecl {
        name: String,
        initializer: Option<Expr>,
        is_const: bool,
        span: Span,
    },

    /// Function declarations
    FnDecl {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        span: Span,
    },

    /// Struct declarations
    StructDecl {
        name: String,
        fields: Vec<String>,
        methods: Vec<Stmt>, // FnDecl statements
        span: Span,
    },

    /// If statements
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        elif_branches: Vec<(Expr, Vec<Stmt>)>,
        else_branch: Option<Vec<Stmt>>,
        span: Span,
    },

    /// While loops
    While {
        condition: Expr,
        body: Vec<Stmt>,
        span: Span,
    },

    /// For loops
    For {
        variable: String,
        iterable: Expr,
        body: Vec<Stmt>,
        span: Span,
    },

    /// Infinite loops
    Loop {
        body: Vec<Stmt>,
        span: Span,
    },

    /// Return statements
    Return {
        value: Option<Expr>,
        span: Span,
    },

    /// Break statements
    Break {
        span: Span,
    },

    /// Continue statements
    Continue {
        span: Span,
    },

    /// Block statements
    Block {
        statements: Vec<Stmt>,
        span: Span,
    },
}

impl AstNode for Stmt {
    fn span(&self) -> &Span {
        match self {
            Stmt::Expression { span, .. } => span,
            Stmt::VarDecl { span, .. } => span,
            Stmt::FnDecl { span, .. } => span,
            Stmt::StructDecl { span, .. } => span,
            Stmt::If { span, .. } => span,
            Stmt::While { span, .. } => span,
            Stmt::For { span, .. } => span,
            Stmt::Loop { span, .. } => span,
            Stmt::Return { span, .. } => span,
            Stmt::Break { span, .. } => span,
            Stmt::Continue { span, .. } => span,
            Stmt::Block { span, .. } => span,
        }
    }
}