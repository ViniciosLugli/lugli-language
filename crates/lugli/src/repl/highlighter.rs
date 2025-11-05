use rustyline::highlight::Highlighter;
use std::borrow::Cow;

pub struct LugliHighlighter;

impl LugliHighlighter {
    pub fn new() -> Self { Self }
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
