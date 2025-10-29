use crate::{AstNode, Expr, NodeId};
use lugli_common::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct StructDeclData {
    pub name: String,
    pub fields: Vec<(String, Option<Expr>)>,
    pub methods: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfData {
    pub condition: Expr,
    pub then_branch: Vec<Stmt>,
    pub elif_branches: Vec<(Expr, Vec<Stmt>)>,
    pub else_branch: Option<Vec<Stmt>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression {
        id: NodeId,
        expr: Expr,
    },

    VarDecl {
        id: NodeId,
        name: String,
        initializer: Option<Expr>,
        is_const: bool,
    },

    FnDecl {
        id: NodeId,
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },

    StructDecl {
        id: NodeId,
        data: Box<StructDeclData>,
    },

    If {
        id: NodeId,
        data: Box<IfData>,
    },

    While {
        id: NodeId,
        condition: Expr,
        body: Vec<Stmt>,
    },

    For {
        id: NodeId,
        variable: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },

    Loop {
        id: NodeId,
        body: Vec<Stmt>,
    },

    Return {
        id: NodeId,
        value: Option<Expr>,
    },

    Break {
        id: NodeId,
    },

    Continue {
        id: NodeId,
    },

    Block {
        id: NodeId,
        statements: Vec<Stmt>,
    },

    Import {
        id: NodeId,
        module_path: Vec<String>,
        items: Option<Vec<String>>,
        alias: Option<String>,
    },

    Export {
        id: NodeId,
        item: Box<Stmt>,
    },
}

impl Stmt {
    pub fn id(&self) -> NodeId {
        match self {
            Stmt::Expression { id, .. } => *id,
            Stmt::VarDecl { id, .. } => *id,
            Stmt::FnDecl { id, .. } => *id,
            Stmt::StructDecl { id, .. } => *id,
            Stmt::If { id, .. } => *id,
            Stmt::While { id, .. } => *id,
            Stmt::For { id, .. } => *id,
            Stmt::Loop { id, .. } => *id,
            Stmt::Return { id, .. } => *id,
            Stmt::Break { id, .. } => *id,
            Stmt::Continue { id, .. } => *id,
            Stmt::Block { id, .. } => *id,
            Stmt::Import { id, .. } => *id,
            Stmt::Export { id, .. } => *id,
        }
    }
}

impl AstNode for Stmt {
    fn span(&self) -> &Span {
        panic!("AstNode::span() called on Stmt after NodeId migration. Use SpanMap instead.")
    }
}
