use logos::Logos;
use lugli_common::Span;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, span: Span) -> Self {
        Self {
            kind,
            lexeme,
            span,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:?}({})", self.kind, self.lexeme) }
}

#[derive(Logos, Debug, Clone, PartialEq)]
pub enum TokenKind {
    #[regex(r"-?[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())]
    Number(f64),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice().to_string())]
    String(String),

    #[regex(r#"f"([^"\\]|\\.)*""#, |lex| lex.slice().to_string())]
    FString(String),

    #[token("true")]
    True,

    #[token("false")]
    False,

    #[token("null")]
    Null,

    // Function and control flow keywords
    #[token("fn")]
    Fn,

    #[token("let")]
    Let,

    #[token("mut")]
    Mut,

    #[token("const")]
    Const,

    #[token("if")]
    If,

    #[token("elif")]
    Elif,

    #[token("else")]
    Else,

    #[token("while")]
    While,

    #[token("for")]
    For,

    #[token("in")]
    In,

    #[token("not")]
    Not,

    #[token("loop")]
    Loop,

    #[token("break")]
    Break,

    #[token("continue")]
    Continue,

    #[token("return")]
    Return,

    // Type and structure keywords
    #[token("struct")]
    Struct,

    #[token("enum")]
    Enum,

    #[token("impl")]
    Impl,

    #[token("trait")]
    Trait,

    #[token("type")]
    Type,

    #[token("class")]
    Class,

    #[token("abstract")]
    Abstract,

    #[token("where")]
    Where,

    #[token("self")]
    SelfKeyword,

    // Pattern matching
    #[token("match")]
    Match,

    // Error handling
    #[token("try")]
    Try,

    #[token("catch")]
    Catch,

    #[token("throw")]
    Throw,

    #[token("finally")]
    Finally,

    #[token("panic")]
    Panic,

    #[token("assert")]
    Assert,

    #[token("debug_assert")]
    DebugAssert,

    // Async/await
    #[token("async")]
    Async,

    #[token("await")]
    Await,

    // Module system
    #[token("import")]
    Import,

    #[token("export")]
    Export,

    #[token("from")]
    From,

    #[token("as")]
    As,

    #[token("pub")]
    Pub,

    // Option and Result keywords
    #[token("Some")]
    Some,

    #[token("None")]
    None,

    #[token("Ok")]
    Ok,

    #[token("Err")]
    Err,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Arithmetic operators
    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("//")]
    IntegerDivision,

    #[token("%")]
    Percent,

    #[token("**")]
    Power,

    // Assignment operators
    #[token("=")]
    Equal,

    #[token("+=")]
    PlusEqual,

    #[token("-=")]
    MinusEqual,

    #[token("*=")]
    StarEqual,

    #[token("/=")]
    SlashEqual,

    #[token("%=")]
    PercentEqual,

    #[token("**=")]
    PowerEqual,

    // Increment/decrement
    #[token("++")]
    Increment,

    #[token("--")]
    Decrement,

    // Comparison operators
    #[token("==")]
    EqualEqual,

    #[token("!=")]
    BangEqual,

    #[token("<")]
    Less,

    #[token("<=")]
    LessEqual,

    #[token(">")]
    Greater,

    #[token(">=")]
    GreaterEqual,

    // Logical operators
    #[token("&&")]
    And,

    #[token("||")]
    Or,

    #[token("!")]
    Bang,

    // Bitwise operators
    #[token("&")]
    Ampersand,

    #[token("|")]
    Pipe,

    #[token("^")]
    Caret,

    #[token("~")]
    Tilde,

    #[token("<<")]
    LeftShift,

    #[token(">>")]
    RightShift,

    // Bitwise assignment operators
    #[token("&=")]
    AmpersandEqual,

    #[token("|=")]
    PipeEqual,

    #[token("^=")]
    CaretEqual,

    #[token("<<=")]
    LeftShiftEqual,

    #[token(">>=")]
    RightShiftEqual,

    // Special operators
    #[token("->")]
    Arrow,

    #[token("::")]
    DoubleColon,

    #[token("..")]
    DotDot,

    #[token("..=")]
    DotDotEqual,

    #[token("=>")]
    FatArrow,

    #[token("?")]
    Question,

    #[token("??")]
    DoubleQuestion,

    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    #[token("{")]
    LeftBrace,

    #[token("}")]
    RightBrace,

    #[token("[")]
    LeftBracket,

    #[token("]")]
    RightBracket,

    // Delimiters
    #[token(",")]
    Comma,

    #[token(".")]
    Dot,

    #[token(":")]
    Colon,

    #[token(";")]
    Semicolon,

    // Comments and whitespace
    #[regex(r"#[^\n]*", logos::skip)]
    #[regex(r"[ \t\f]+", logos::skip)]
    #[token("\n")]
    Newline,

    Eof,
}
