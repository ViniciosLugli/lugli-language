use lugli_common::{Span, StringPool, Value};
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub enum Instruction {
    Constant(usize),
    Load(usize),
    Store(usize),
    Add,
    Subtract,
    Multiply,
    Divide,
    IntegerDivide,
    Modulo,
    Power,
    Negate,
    Not,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    And,
    Or,
    Print,
    Pop,
    Dup,
    Jump(usize),
    JumpIfFalse(usize),
    Loop(usize),
    Call(u8),
    CallMethod(usize, u8),
    Return,
    GetProperty(usize),
    SetProperty(usize),
    MakeList(usize),
    MakeDict(usize),
    GetIndex,
    SetIndex,
    ToString,
    LoadGlobal(usize),
    StoreGlobal(usize),
    DefineFunction(usize),
    MakeClosure { function_index: usize, capture_indices: Vec<usize> },
    LoadUpvalue(usize),
    StoreUpvalue(usize),
    // Module system - these store directly in globals, no stack effect
    ImportModule { module_idx: usize, bind_name: String }, // import X -> binds module to name
    ImportFrom { module_idx: usize, names: Vec<String> },  // from X import Y -> binds items to names
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file_path: String,
    pub span: Span,
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub fn new(file_path: String, span: Span, line: usize, column: usize) -> Self {
        Self {
            file_path,
            span,
            line,
            column,
        }
    }

    pub fn unknown() -> Self {
        Self {
            file_path: "<unknown>".to_string(),
            span: Span::new(0, 0),
            line: 0,
            column: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Bytecode {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
    pub source_map: Vec<SourceLocation>,
    pub source_code: Option<String>,
    pub string_pool: RefCell<StringPool>,
}

impl Bytecode {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            source_map: Vec::new(),
            source_code: None,
            string_pool: RefCell::new(StringPool::new()),
        }
    }

    pub fn with_source(source_code: String) -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            source_map: Vec::new(),
            source_code: Some(source_code),
            string_pool: RefCell::new(StringPool::new()),
        }
    }

    pub fn with_pool(pool: StringPool) -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            source_map: Vec::new(),
            source_code: None,
            string_pool: RefCell::new(pool),
        }
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn emit(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
        self.source_map.push(SourceLocation::unknown());
    }

    pub fn emit_with_location(&mut self, instruction: Instruction, location: SourceLocation) {
        self.instructions.push(instruction);
        self.source_map.push(location);
    }

    pub fn get_location(&self, ip: usize) -> Option<&SourceLocation> { self.source_map.get(ip) }
}

impl Default for Bytecode {
    fn default() -> Self { Self::new() }
}
