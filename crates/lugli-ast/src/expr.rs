use crate::{AstNode, Stmt};
use lugli_common::Span;
use lugli_lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal { value: LiteralValue, span: Span },

    Identifier { name: String, span: Span },

    Binary { left: Box<Expr>, operator: Token, right: Box<Expr>, span: Span },

    Unary { operator: Token, operand: Box<Expr>, span: Span },

    Call { callee: Box<Expr>, arguments: Vec<Expr>, span: Span },

    Get { object: Box<Expr>, name: String, span: Span },

    Set { object: Box<Expr>, name: String, value: Box<Expr>, span: Span },

    List { elements: Vec<Expr>, span: Span },

    Dict { pairs: Vec<(Expr, Expr)>, span: Span },

    Index { object: Box<Expr>, index: Box<Expr>, span: Span },

    FString { parts: Vec<FStringPart>, span: Span },

    Function { params: Vec<String>, body: Vec<Stmt>, span: Span },

    ListComprehension { element: Box<Expr>, variable: String, iterable: Box<Expr>, condition: Option<Box<Expr>>, span: Span },

    Match { value: Box<Expr>, arms: Vec<MatchArm>, span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(LiteralValue),
    Identifier(String),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FStringPart {
    Text(String),
    Expression(Box<Expr>),
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
            Expr::Literal {
                span, ..
            } => span,
            Expr::Identifier {
                span, ..
            } => span,
            Expr::Binary {
                span, ..
            } => span,
            Expr::Unary {
                span, ..
            } => span,
            Expr::Call {
                span, ..
            } => span,
            Expr::Get {
                span, ..
            } => span,
            Expr::Set {
                span, ..
            } => span,
            Expr::List {
                span, ..
            } => span,
            Expr::Dict {
                span, ..
            } => span,
            Expr::Index {
                span, ..
            } => span,
            Expr::FString {
                span, ..
            } => span,
            Expr::Function {
                span, ..
            } => span,
            Expr::ListComprehension {
                span, ..
            } => span,
            Expr::Match {
                span, ..
            } => span,
        }
    }
}
