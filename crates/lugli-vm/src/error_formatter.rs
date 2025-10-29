use colored::*;
use lugli_common::{ErrorCode, LugliError, SourceContext};

pub struct ErrorFormatter {
    use_colors: bool,
}

impl ErrorFormatter {
    pub fn new() -> Self {
        Self {
            use_colors: true,
        }
    }

    pub fn without_colors() -> Self {
        Self {
            use_colors: false,
        }
    }

    pub fn format(&self, error: &LugliError) -> String {
        let mut output = String::new();

        match error {
            LugliError::Runtime(data) => {
                self.format_runtime_error(
                    &mut output,
                    &data.message,
                    data.code,
                    data.context.as_ref(),
                    data.suggestion.as_deref(),
                    &data.stack_trace,
                );
            }
            LugliError::Type(data) => {
                self.format_type_error(&mut output, &data.expected, &data.found, data.code, data.context.as_ref());
            }
            LugliError::UndefinedVariable(data) => {
                self.format_undefined_variable(&mut output, &data.name, data.context.as_ref(), data.suggestion.as_deref());
            }
            LugliError::DivisionByZero {
                context,
            } => {
                self.format_division_by_zero(&mut output, context.as_ref());
            }
            LugliError::IndexOutOfBounds {
                index,
                length,
                context,
            } => {
                self.format_index_error(&mut output, *index, *length, context.as_ref());
            }
            _ => {
                output.push_str(&format!("{}", error));
            }
        }

        output
    }

    fn format_runtime_error(
        &self,
        output: &mut String,
        message: &str,
        code: ErrorCode,
        context: Option<&SourceContext>,
        suggestion: Option<&str>,
        stack_trace: &str,
    ) {
        let header = if self.use_colors {
            format!("{} [{}]: {}: {}", "Error".red().bold(), code.as_str().yellow(), "Runtime Error".red(), message)
        } else {
            format!("Error [{}]: Runtime Error: {}", code.as_str(), message)
        };

        output.push_str(&header);
        output.push('\n');

        if let Some(ctx) = context {
            self.format_source_context(output, ctx);
        }

        if let Some(help) = suggestion {
            output.push('\n');
            if self.use_colors {
                output.push_str(&format!("{}: {}", "Help".cyan().bold(), help));
            } else {
                output.push_str(&format!("Help: {}", help));
            }
            output.push('\n');
        }

        if !stack_trace.is_empty() {
            output.push_str(stack_trace);
        }
    }

    fn format_type_error(&self, output: &mut String, expected: &str, found: &str, code: ErrorCode, context: Option<&SourceContext>) {
        let header = if self.use_colors {
            format!(
                "{} [{}]: {}: expected {}, found {}",
                "Error".red().bold(),
                code.as_str().yellow(),
                "Type Error".red(),
                expected.green(),
                found.red()
            )
        } else {
            format!("Error [{}]: Type Error: expected {}, found {}", code.as_str(), expected, found)
        };

        output.push_str(&header);
        output.push('\n');

        if let Some(ctx) = context {
            self.format_source_context(output, ctx);
        }
    }

    fn format_undefined_variable(&self, output: &mut String, name: &str, context: Option<&SourceContext>, suggestion: Option<&str>) {
        let header = if self.use_colors {
            format!("{} [{}]: {}: Undefined variable '{}'", "Error".red().bold(), "E003".yellow(), "Runtime Error".red(), name.cyan())
        } else {
            format!("Error [E003]: Runtime Error: Undefined variable '{}'", name)
        };

        output.push_str(&header);
        output.push('\n');

        if let Some(ctx) = context {
            self.format_source_context(output, ctx);
        }

        if let Some(help) = suggestion {
            output.push('\n');
            if self.use_colors {
                output.push_str(&format!("{}: {}", "Help".cyan().bold(), help));
            } else {
                output.push_str(&format!("Help: {}", help));
            }
            output.push('\n');
        }
    }

    fn format_division_by_zero(&self, output: &mut String, context: Option<&SourceContext>) {
        let header = if self.use_colors {
            format!("{} [{}]: {}: Division by zero", "Error".red().bold(), "E004".yellow(), "Runtime Error".red())
        } else {
            "Error [E004]: Runtime Error: Division by zero".to_string()
        };

        output.push_str(&header);
        output.push('\n');

        if let Some(ctx) = context {
            self.format_source_context(output, ctx);
        }

        output.push('\n');
        if self.use_colors {
            output.push_str(&format!("{}: Ensure the divisor is not zero before division", "Help".cyan().bold()));
        } else {
            output.push_str("Help: Ensure the divisor is not zero before division");
        }
        output.push('\n');
    }

    fn format_index_error(&self, output: &mut String, index: i64, length: usize, context: Option<&SourceContext>) {
        let header = if self.use_colors {
            format!(
                "{} [{}]: {}: Index {} out of bounds (length: {})",
                "Error".red().bold(),
                "E005".yellow(),
                "Runtime Error".red(),
                index.to_string().cyan(),
                length
            )
        } else {
            format!("Error [E005]: Runtime Error: Index {} out of bounds (length: {})", index, length)
        };

        output.push_str(&header);
        output.push('\n');

        if let Some(ctx) = context {
            self.format_source_context(output, ctx);
        }

        output.push('\n');
        if self.use_colors {
            output.push_str(&format!(
                "{}: Valid indices are 0 to {} (or -{} to -1 for negative indexing)",
                "Help".cyan().bold(),
                if length > 0 { length - 1 } else { 0 },
                length
            ));
        } else {
            output.push_str(&format!(
                "Help: Valid indices are 0 to {} (or -{} to -1 for negative indexing)",
                if length > 0 { length - 1 } else { 0 },
                length
            ));
        }
        output.push('\n');
    }

    fn format_source_context(&self, output: &mut String, context: &SourceContext) {
        let location = if self.use_colors {
            format!("  {} {}:{}:{}", "-->".blue().bold(), context.file_path, context.line, context.column)
        } else {
            format!("  --> {}:{}:{}", context.file_path, context.line, context.column)
        };

        output.push_str(&location);
        output.push('\n');

        if let Some(source_line) = &context.source_line {
            let line_num = format!("{}", context.line);
            let padding = " ".repeat(line_num.len());

            if self.use_colors {
                output.push_str(&format!(" {} |\n", padding.blue()));
                output.push_str(&format!(" {} | {}\n", line_num.blue().bold(), source_line));

                let caret_line = self.generate_caret_line(source_line, context.column, context.span.len());
                output.push_str(&format!(" {} | {}\n", padding.blue(), caret_line.red().bold()));
            } else {
                output.push_str(&format!(" {} |\n", padding));
                output.push_str(&format!(" {} | {}\n", line_num, source_line));

                let caret_line = self.generate_caret_line(source_line, context.column, context.span.len());
                output.push_str(&format!(" {} | {}\n", padding, caret_line));
            }
        }
    }

    fn generate_caret_line(&self, source_line: &str, column: usize, span_len: usize) -> String {
        let mut caret_line = String::new();

        for (i, c) in source_line.chars().enumerate() {
            if i < column - 1 {
                if c == '\t' {
                    caret_line.push('\t');
                } else {
                    caret_line.push(' ');
                }
            } else if i < column - 1 + span_len {
                caret_line.push('^');
            } else {
                break;
            }
        }

        if caret_line.is_empty() || !caret_line.contains('^') {
            caret_line.push('^');
        }

        caret_line
    }
}

impl Default for ErrorFormatter {
    fn default() -> Self { Self::new() }
}

pub fn suggest_similar_name(name: &str, available: &[&str]) -> Option<String> {
    if available.is_empty() {
        return None;
    }

    let mut best_match: Option<(&str, usize)> = None;

    for candidate in available {
        let distance = levenshtein_distance(name, candidate);
        if distance <= 2 {
            if let Some((_, best_distance)) = best_match {
                if distance < best_distance {
                    best_match = Some((candidate, distance));
                }
            } else {
                best_match = Some((candidate, distance));
            }
        }
    }

    best_match.map(|(name, _)| format!("Did you mean '{}'?", name))
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1).min(matrix[i][j - 1] + 1).min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[a_len][b_len]
}

#[cfg(test)]
mod tests {
    use super::*;
    use lugli_common::Span;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("foo", "foo"), 0);
        assert_eq!(levenshtein_distance("foo", "foe"), 1);
        assert_eq!(levenshtein_distance("foo", "bar"), 3);
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    }

    #[test]
    fn test_suggest_similar_name() {
        let available = vec!["foo", "bar", "baz"];
        assert!(suggest_similar_name("foe", &available).is_some());
        assert!(suggest_similar_name("bar", &available).is_some());
        assert!(suggest_similar_name("completely_different", &available).is_none());
    }

    #[test]
    fn test_format_division_by_zero_no_color() {
        let formatter = ErrorFormatter::without_colors();
        let error = LugliError::division_by_zero();
        let formatted = formatter.format(&error);
        assert!(formatted.contains("E004"));
        assert!(formatted.contains("Division by zero"));
    }

    #[test]
    fn test_format_undefined_variable_no_color() {
        let formatter = ErrorFormatter::without_colors();
        let error = LugliError::undefined_variable("foo");
        let formatted = formatter.format(&error);
        assert!(formatted.contains("E003"));
        assert!(formatted.contains("foo"));
    }

    #[test]
    fn test_format_with_context() {
        let formatter = ErrorFormatter::without_colors();
        let context = SourceContext::new("test.lg".to_string(), 5, 10, Span::new(45, 48)).with_source("    let x = foo + 1".to_string());

        let error = LugliError::undefined_variable_with_context("foo", context, Some("Did you mean 'for'?".to_string()));

        let formatted = formatter.format(&error);
        assert!(formatted.contains("test.lg:5:10"));
        assert!(formatted.contains("foo"));
        assert!(formatted.contains("^^^"));
        assert!(formatted.contains("Did you mean 'for'?"));
    }
}
