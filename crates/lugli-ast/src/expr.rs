use lugli_common::Span;
use lugli_lexer::Token;
use crate::AstNode;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal {
        value: LiteralValue,
        span: Span,
    },

    Identifier {
        name: String,
        span: Span,
    },

    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
        span: Span,
    },

    Unary {
        operator: Token,
        operand: Box<Expr>,
        span: Span,
    },

    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
        span: Span,
    },

    Get {
        object: Box<Expr>,
        name: String,
        span: Span,
    },

    Set {
        object: Box<Expr>,
        name: String,
        value: Box<Expr>,
        span: Span,
    },

    List {
        elements: Vec<Expr>,
        span: Span,
    },

    Dict {
        pairs: Vec<(Expr, Expr)>,
        span: Span,
    },

    Index {
        object: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
}

impl AstNode for Expr {
    fn span(&self) -> &Span {
        match self {
            Expr::Literal { span, .. } => span,
            Expr::Identifier { span, .. } => span,
            Expr::Binary { span, .. } => span,
            Expr::Unary { span, .. } => span,
            Expr::Call { span, .. } => span,
            Expr::Get { span, .. } => span,
            Expr::Set { span, .. } => span,
            Expr::List { span, .. } => span,
            Expr::Dict { span, .. } => span,
            Expr::Index { span, .. } => span,
        }
    }
}