use lugli_lexer::TokenKind;
use rustyline::highlight::Highlighter;
use std::borrow::Cow;

pub struct LugliHighlighter;

impl LugliHighlighter {
    pub fn new() -> Self {
        Self
    }

    fn highlight_token(token_kind: &TokenKind) -> Option<String> {
        match token_kind {
            // Keywords
            TokenKind::Let | TokenKind::Mut | TokenKind::Const | TokenKind::Fn
            | TokenKind::Struct | TokenKind::Impl | TokenKind::If | TokenKind::Elif
            | TokenKind::Else | TokenKind::Match | TokenKind::For | TokenKind::While
            | TokenKind::Loop | TokenKind::Break | TokenKind::Continue | TokenKind::Return
            | TokenKind::Import | TokenKind::From | TokenKind::As => None, // Keep keywords plain for readability

            // Literals
            TokenKind::Number(_) => None,
            TokenKind::String(_) => None,
            TokenKind::True | TokenKind::False => Some("bright_blue".to_string()),
            TokenKind::Null => Some("bright_black".to_string()),

            // Identifiers - don't color, keep natural
            TokenKind::Identifier(_) => None,

            // Operators - subtle color
            TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash
            | TokenKind::Percent | TokenKind::Equal | TokenKind::BangEqual
            | TokenKind::Less | TokenKind::LessEqual | TokenKind::Greater
            | TokenKind::GreaterEqual | TokenKind::And | TokenKind::Or | TokenKind::Bang
            | TokenKind::PlusEqual | TokenKind::MinusEqual | TokenKind::StarEqual
            | TokenKind::SlashEqual => None,

            _ => None,
        }
    }
}

impl Highlighter for LugliHighlighter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // For now, return the line as-is for better readability
        // Full syntax highlighting can be overwhelming in a REPL
        Cow::Borrowed(line)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: rustyline::highlight::CmdKind) -> bool {
        // Disable character-by-character highlighting for better performance
        false
    }
}
