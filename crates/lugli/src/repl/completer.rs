use rustyline::{
    Context,
    completion::{Completer, Pair},
};
use std::collections::HashSet;

pub struct LugliCompleter {
    keywords: Vec<String>,
    builtins: Vec<String>,
    variables: Vec<String>,
}

impl LugliCompleter {
    pub fn new() -> Self {
        let keywords = vec![
            // Keywords
            "let", "mut", "const", "fn", "struct", "impl", "if", "elif", "else", "match", "for", "while", "loop", "break", "continue", "return",
            "import", "from", "as", "true", "false", "null", // Type hints
            "num", "str", "bool", "list", "dict",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let builtins = vec![
            // Core functions
            "print",
            "println",
            "input",
            "len",
            "str",
            "int",
            "float",
            "bool",
            "type",
            "range",
            "enumerate",
            "zip",
            "map",
            "filter",
            "reduce",
            "all",
            "any",
            "sum",
            "min",
            "max",
            "sorted",
            "reversed",
            // Math functions
            "abs",
            "pow",
            "round",
            "floor",
            "ceil",
            "sqrt",
            // String methods
            "upper",
            "lower",
            "split",
            "join",
            "replace",
            "trim",
            "contains",
            // List methods
            "push",
            "pop",
            "insert",
            "remove",
            "clear",
            "extend",
            // Dict methods
            "keys",
            "values",
            "items",
            "get",
            "has_key",
            // Macros
            "format!",
            "import!",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        Self {
            keywords,
            builtins,
            variables: Vec::new(),
        }
    }

    fn get_candidates(&self, prefix: &str) -> Vec<String> {
        let prefix_lower = prefix.to_lowercase();
        let mut candidates = HashSet::new();

        // Add matching keywords
        for kw in &self.keywords {
            if kw.to_lowercase().starts_with(&prefix_lower) {
                candidates.insert(kw.clone());
            }
        }

        // Add matching builtins
        for builtin in &self.builtins {
            if builtin.to_lowercase().starts_with(&prefix_lower) {
                candidates.insert(builtin.clone());
            }
        }

        // Add matching variables
        for var in &self.variables {
            if var.to_lowercase().starts_with(&prefix_lower) {
                candidates.insert(var.clone());
            }
        }

        let mut result: Vec<_> = candidates.into_iter().collect();
        result.sort();
        result
    }

    fn find_completion_start(line: &str, pos: usize) -> usize {
        let before_cursor = &line[..pos];

        // Find the start of the current word

        before_cursor.rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '!' && c != '?').map(|i| i + 1).unwrap_or(0)
    }
}

impl Completer for LugliCompleter {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        if line.is_empty() || pos == 0 {
            return Ok((0, vec![]));
        }

        let start = Self::find_completion_start(line, pos);
        let prefix = &line[start..pos];

        if prefix.is_empty() {
            return Ok((start, vec![]));
        }

        let candidates = self
            .get_candidates(prefix)
            .into_iter()
            .map(|c| Pair {
                display: c.clone(),
                replacement: c,
            })
            .collect();

        Ok((start, candidates))
    }
}
