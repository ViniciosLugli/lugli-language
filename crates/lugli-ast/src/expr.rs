use crate::{AstNode, NodeId, Stmt};
use lugli_common::Span;
use lugli_lexer::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub struct ListComprehensionData {
    pub element: Expr,
    pub variable: String,
    pub iterable: Expr,
    pub condition: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallData {
    pub callee: Expr,
    pub arguments: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal {
        id: NodeId,
        value: LiteralValue,
    },

    Identifier {
        id: NodeId,
        name: String,
    },

    Binary {
        id: NodeId,
        left: Box<Expr>,
        operator: TokenKind,
        right: Box<Expr>,
    },

    Unary {
        id: NodeId,
        operator: TokenKind,
        operand: Box<Expr>,
    },

    Call {
        id: NodeId,
        data: Box<CallData>,
    },

    Get {
        id: NodeId,
        object: Box<Expr>,
        name: String,
    },

    Set {
        id: NodeId,
        object: Box<Expr>,
        name: String,
        value: Box<Expr>,
    },

    List {
        id: NodeId,
        elements: Vec<Expr>,
    },

    Dict {
        id: NodeId,
        pairs: Vec<(Expr, Expr)>,
    },

    Index {
        id: NodeId,
        object: Box<Expr>,
        index: Box<Expr>,
    },

    FString {
        id: NodeId,
        parts: Box<Vec<FStringPart>>,
    },

    Function {
        id: NodeId,
        params: Vec<String>,
        body: Vec<Stmt>,
    },

    ListComprehension {
        id: NodeId,
        data: Box<ListComprehensionData>,
    },

    Match {
        id: NodeId,
        value: Box<Expr>,
        arms: Vec<MatchArm>,
    },
}

impl Expr {
    pub fn id(&self) -> NodeId {
        match self {
            Expr::Literal { id, .. } => *id,
            Expr::Identifier { id, .. } => *id,
            Expr::Binary { id, .. } => *id,
            Expr::Unary { id, .. } => *id,
            Expr::Call { id, .. } => *id,
            Expr::Get { id, .. } => *id,
            Expr::Set { id, .. } => *id,
            Expr::List { id, .. } => *id,
            Expr::Dict { id, .. } => *id,
            Expr::Index { id, .. } => *id,
            Expr::FString { id, .. } => *id,
            Expr::Function { id, .. } => *id,
            Expr::ListComprehension { id, .. } => *id,
            Expr::Match { id, .. } => *id,
        }
    }
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
        panic!("AstNode::span() called on Expr after NodeId migration. Use SpanMap instead.")
    }
}
