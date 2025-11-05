use colored::Colorize;
use rustyline::highlight::Highlighter;
use std::borrow::Cow;

pub struct LugliHighlighter {
    keywords: Vec<&'static str>,
}

impl LugliHighlighter {
    pub fn new() -> Self {
        Self {
            keywords: vec![
                "let", "mut", "const", "fn", "struct", "impl", "if", "elif", "else", "match", "for", "while", "loop", "break", "continue", "return",
                "import", "from", "as", "true", "false", "null",
            ],
        }
    }

    fn is_keyword(&self, word: &str) -> bool { self.keywords.contains(&word) }
}

impl Highlighter for LugliHighlighter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        let mut result = String::new();
        let mut chars = line.chars().peekable();
        let mut current_word = String::new();

        while let Some(ch) = chars.next() {
            match ch {
                // String literals
                '"' => {
                    if !current_word.is_empty() {
                        result.push_str(&self.highlight_word(&current_word));
                        current_word.clear();
                    }
                    let mut string_lit = String::from("\"");
                    while let Some(&next_ch) = chars.peek() {
                        chars.next();
                        string_lit.push(next_ch);
                        if next_ch == '"' {
                            break;
                        }
                        if next_ch == '\\' {
                            if let Some(&escaped) = chars.peek() {
                                chars.next();
                                string_lit.push(escaped);
                            }
                        }
                    }
                    result.push_str(&string_lit.green().to_string());
                }
                // Comments
                '#' => {
                    if !current_word.is_empty() {
                        result.push_str(&self.highlight_word(&current_word));
                        current_word.clear();
                    }
                    let mut comment = String::from("#");
                    for next_ch in chars.by_ref() {
                        comment.push(next_ch);
                    }
                    result.push_str(&comment.bright_black().to_string());
                }
                // Numbers
                '0'..='9' if current_word.is_empty() || current_word.chars().all(|c| c.is_numeric() || c == '.') => {
                    current_word.push(ch);
                    // Look ahead for more digits/decimal point
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_numeric() || next_ch == '.' {
                            current_word.push(next_ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    result.push_str(&current_word.yellow().to_string());
                    current_word.clear();
                }
                // Word boundaries
                ' ' | '\t' | '(' | ')' | '{' | '}' | '[' | ']' | ',' | ';' | ':' | '+' | '-' | '*' | '/' | '=' | '!' | '<' | '>' | '&' | '|' => {
                    if !current_word.is_empty() {
                        result.push_str(&self.highlight_word(&current_word));
                        current_word.clear();
                    }
                    result.push(ch);
                }
                // Regular characters
                _ => {
                    current_word.push(ch);
                }
            }
        }

        // Flush remaining word
        if !current_word.is_empty() {
            result.push_str(&self.highlight_word(&current_word));
        }

        Cow::Owned(result)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: rustyline::highlight::CmdKind) -> bool { true }
}

impl LugliHighlighter {
    fn highlight_word(&self, word: &str) -> String { if self.is_keyword(word) { word.blue().to_string() } else { word.to_string() } }
}
