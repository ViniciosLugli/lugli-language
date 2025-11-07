use logos::Logos;
use lugli_common::{Span, StringId};
use std::fmt;

// Placeholder StringId used by logos (will be replaced during lexing)
const PLACEHOLDER_STRING_ID: StringId = StringId::from_u32(0);

// Custom f-string lexer using Python-inspired state machine approach
// Note: logos has already consumed "f\"" or "f'", so we need to determine the quote type
fn lex_fstring(lex: &mut logos::Lexer<TokenKind>) -> Option<StringId> {
    // Determine which quote was used by looking at the matched text
    let slice = lex.slice();
    let quote = slice.chars().last()?; // Get the quote character (last char of "f\"" or "f'")

    let remainder = lex.remainder();
    let mut chars = remainder.chars();

    let mut brace_depth = 0;
    let mut in_string = false;
    let mut string_delimiter = None;
    let mut escape_next = false;
    let mut consumed = 0;

    while let Some(ch) = chars.next() {
        consumed += ch.len_utf8();

        if escape_next {
            escape_next = false;
            continue;
        }

        if ch == '\\' {
            escape_next = true;
            continue;
        }

        // Handle quotes inside interpolations
        if ch == '"' || ch == '\'' {
            if in_string && Some(ch) == string_delimiter {
                // Close the nested string
                in_string = false;
                string_delimiter = None;
            } else if !in_string && brace_depth > 0 {
                // Open a nested string inside interpolation
                in_string = true;
                string_delimiter = Some(ch);
            } else if !in_string && brace_depth == 0 && ch == quote {
                // This is the closing quote of the f-string
                lex.bump(consumed);
                return Some(PLACEHOLDER_STRING_ID);
            }
            continue;
        }

        // Track braces for interpolations (only when not in nested strings)
        if !in_string {
            if ch == '{' {
                // Check for escaped brace {{ (only when not inside interpolation)
                if brace_depth == 0 && chars.clone().next() == Some('{') {
                    chars.next(); // consume second {
                    consumed += 1;
                } else {
                    brace_depth += 1;
                }
            } else if ch == '}' {
                // Check for escaped brace }} (only when not inside interpolation)
                if brace_depth == 0 && chars.clone().next() == Some('}') {
                    chars.next(); // consume second }
                    consumed += 1;
                } else if brace_depth > 0 {
                    brace_depth -= 1;
                }
            }
        }
    }

    // Unterminated f-string - consume everything to EOF and let parser handle the error
    // This allows better error messages from the parser
    lex.bump(consumed);
    Some(PLACEHOLDER_STRING_ID)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self {
            kind,
            span,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            TokenKind::Number(n) => write!(f, "Number({})", n),
            TokenKind::String(id) => write!(f, "String({:?})", id),
            TokenKind::FString(id) => write!(f, "FString({:?})", id),
            TokenKind::Identifier(id) => write!(f, "Identifier({:?})", id),
            other => write!(f, "{:?}", other),
        }
    }
}

#[derive(Logos, Debug, Clone, PartialEq)]
pub enum TokenKind {
    #[regex(r"-?[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())]
    Number(f64),

    #[regex(r#""([^"\\]|\\.)*""#, |_| PLACEHOLDER_STRING_ID)]
    #[regex(r#"'([^'\\]|\\.)*'"#, |_| PLACEHOLDER_STRING_ID)]
    String(StringId),

    #[token("f\"", lex_fstring)]
    #[token("f'", lex_fstring)]
    FString(StringId),

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

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |_| PLACEHOLDER_STRING_ID)]
    Identifier(StringId),

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
