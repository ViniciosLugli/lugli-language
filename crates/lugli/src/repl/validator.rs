use rustyline::validate::{ValidationContext, ValidationResult, Validator};

pub struct InputValidator;

impl InputValidator {
    pub fn new() -> Self {
        Self
    }

    fn count_unclosed_delimiters(input: &str) -> (i32, i32, i32) {
        let mut parens = 0;
        let mut braces = 0;
        let mut brackets = 0;
        let mut in_string = false;
        let mut in_char = false;
        let mut escape = false;

        for ch in input.chars() {
            if escape {
                escape = false;
                continue;
            }

            if ch == '\\' {
                escape = true;
                continue;
            }

            if in_char {
                if ch == '\'' {
                    in_char = false;
                }
                continue;
            }

            if in_string {
                if ch == '"' {
                    in_string = false;
                }
                continue;
            }

            match ch {
                '"' => in_string = true,
                '\'' => in_char = true,
                '(' => parens += 1,
                ')' => parens -= 1,
                '{' => braces += 1,
                '}' => braces -= 1,
                '[' => brackets += 1,
                ']' => brackets -= 1,
                _ => {}
            }
        }

        (parens, braces, brackets)
    }

    fn needs_more_input(input: &str) -> bool {
        let trimmed = input.trim();

        // Check for unclosed delimiters
        let (parens, braces, brackets) = Self::count_unclosed_delimiters(trimmed);

        if parens > 0 || braces > 0 || brackets > 0 {
            return true;
        }

        // Check if line ends with certain keywords that suggest continuation
        let ends_with_continuation = trimmed.ends_with('{')
            || trimmed.ends_with('(')
            || trimmed.ends_with('[')
            || trimmed.ends_with(',')
            || trimmed.ends_with('\\');

        ends_with_continuation
    }
}

impl Validator for InputValidator {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();

        if Self::needs_more_input(input) {
            Ok(ValidationResult::Incomplete)
        } else {
            Ok(ValidationResult::Valid(None))
        }
    }
}
