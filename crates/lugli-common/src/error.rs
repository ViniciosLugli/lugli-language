use thiserror::Error;
use crate::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    E001,
    E002,
    E003,
    E004,
    E005,
    E006,
    E007,
    E008,
    E009,
    E010,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::E001 => "E001",
            ErrorCode::E002 => "E002",
            ErrorCode::E003 => "E003",
            ErrorCode::E004 => "E004",
            ErrorCode::E005 => "E005",
            ErrorCode::E006 => "E006",
            ErrorCode::E007 => "E007",
            ErrorCode::E008 => "E008",
            ErrorCode::E009 => "E009",
            ErrorCode::E010 => "E010",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceContext {
    pub file_path: String,
    pub line: usize,
    pub column: usize,
    pub source_line: Option<String>,
    pub span: Span,
}

impl SourceContext {
    pub fn new(file_path: String, line: usize, column: usize, span: Span) -> Self {
        Self { file_path, line, column, source_line: None, span }
    }

    pub fn with_source(mut self, source_line: String) -> Self {
        self.source_line = Some(source_line);
        self
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeErrorData {
    pub message: String,
    pub stack_trace: String,
    pub code: ErrorCode,
    pub context: Option<SourceContext>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TypeErrorData {
    pub expected: String,
    pub found: String,
    pub code: ErrorCode,
    pub context: Option<SourceContext>,
}

#[derive(Debug, Clone)]
pub struct UndefinedVariableData {
    pub name: String,
    pub context: Option<SourceContext>,
    pub suggestion: Option<String>,
}

#[derive(Error, Debug, Clone)]
pub enum LugliError {
    #[error("Lexical error: {message} at {span}")]
    Lex { message: String, span: Span },

    #[error("Parse error: {message} at {span}")]
    Parse { message: String, span: Span },

    #[error("Runtime error: {}{}", .0.message, .0.stack_trace)]
    Runtime(Box<RuntimeErrorData>),

    #[error("Type error: expected {}, found {}", .0.expected, .0.found)]
    Type(Box<TypeErrorData>),

    #[error("Undefined variable: {}", .0.name)]
    UndefinedVariable(Box<UndefinedVariableData>),

    #[error("Division by zero")]
    DivisionByZero {
        context: Option<SourceContext>,
    },

    #[error("Index out of bounds: {index} (length: {length})")]
    IndexOutOfBounds {
        index: i64,
        length: usize,
        context: Option<SourceContext>,
    },

    #[error("IO error: {message}")]
    Io { message: String },
}

impl LugliError {
    pub fn lex(message: impl Into<String>, span: Span) -> Self {
        Self::Lex { message: message.into(), span }
    }

    pub fn parse(message: impl Into<String>, span: Span) -> Self {
        Self::Parse { message: message.into(), span }
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime(Box::new(RuntimeErrorData {
            message: message.into(),
            stack_trace: "".to_string(),
            code: ErrorCode::E001,
            context: None,
            suggestion: None,
        }))
    }

    pub fn runtime_with_trace(message: impl Into<String>, trace: Vec<String>) -> Self {
        let stack_trace = if trace.is_empty() {
            "".to_string()
        } else {
            format!("\nStack trace:\n{}", trace.join("\n"))
        };
        Self::Runtime(Box::new(RuntimeErrorData {
            message: message.into(),
            stack_trace,
            code: ErrorCode::E001,
            context: None,
            suggestion: None,
        }))
    }

    pub fn runtime_with_context(
        message: impl Into<String>,
        code: ErrorCode,
        context: SourceContext,
        suggestion: Option<String>
    ) -> Self {
        Self::Runtime(Box::new(RuntimeErrorData {
            message: message.into(),
            stack_trace: "".to_string(),
            code,
            context: Some(context),
            suggestion,
        }))
    }

    pub fn runtime_full(
        message: impl Into<String>,
        code: ErrorCode,
        context: Option<SourceContext>,
        suggestion: Option<String>,
        trace: Vec<String>
    ) -> Self {
        let stack_trace = if trace.is_empty() {
            "".to_string()
        } else {
            format!("\nStack trace:\n{}", trace.join("\n"))
        };
        Self::Runtime(Box::new(RuntimeErrorData {
            message: message.into(),
            stack_trace,
            code,
            context,
            suggestion,
        }))
    }

    pub fn type_error(expected: impl Into<String>, found: impl Into<String>) -> Self {
        Self::Type(Box::new(TypeErrorData {
            expected: expected.into(),
            found: found.into(),
            code: ErrorCode::E002,
            context: None,
        }))
    }

    pub fn type_error_with_context(
        expected: impl Into<String>,
        found: impl Into<String>,
        context: SourceContext
    ) -> Self {
        Self::Type(Box::new(TypeErrorData {
            expected: expected.into(),
            found: found.into(),
            code: ErrorCode::E002,
            context: Some(context),
        }))
    }

    pub fn undefined_variable(name: impl Into<String>) -> Self {
        Self::UndefinedVariable(Box::new(UndefinedVariableData {
            name: name.into(),
            context: None,
            suggestion: None,
        }))
    }

    pub fn undefined_variable_with_context(
        name: impl Into<String>,
        context: SourceContext,
        suggestion: Option<String>
    ) -> Self {
        Self::UndefinedVariable(Box::new(UndefinedVariableData {
            name: name.into(),
            context: Some(context),
            suggestion,
        }))
    }

    pub fn division_by_zero() -> Self {
        Self::DivisionByZero { context: None }
    }

    pub fn division_by_zero_with_context(context: SourceContext) -> Self {
        Self::DivisionByZero { context: Some(context) }
    }

    pub fn index_out_of_bounds(index: i64, length: usize) -> Self {
        Self::IndexOutOfBounds {
            index,
            length,
            context: None,
        }
    }

    pub fn index_out_of_bounds_with_context(
        index: i64,
        length: usize,
        context: SourceContext
    ) -> Self {
        Self::IndexOutOfBounds {
            index,
            length,
            context: Some(context),
        }
    }

    pub fn get_code(&self) -> Option<ErrorCode> {
        match self {
            LugliError::Runtime(data) => Some(data.code),
            LugliError::Type(data) => Some(data.code),
            LugliError::UndefinedVariable(_) => Some(ErrorCode::E003),
            LugliError::DivisionByZero { .. } => Some(ErrorCode::E004),
            LugliError::IndexOutOfBounds { .. } => Some(ErrorCode::E005),
            _ => None,
        }
    }

    pub fn get_context(&self) -> Option<&SourceContext> {
        match self {
            LugliError::Runtime(data) => data.context.as_ref(),
            LugliError::Type(data) => data.context.as_ref(),
            LugliError::UndefinedVariable(data) => data.context.as_ref(),
            LugliError::DivisionByZero { context } => context.as_ref(),
            LugliError::IndexOutOfBounds { context, .. } => context.as_ref(),
            _ => None,
        }
    }
}
