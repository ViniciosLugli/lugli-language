use lugli_common::Value;

#[derive(Debug, Clone)]
pub enum Instruction {
    Constant(usize),
    Load(usize),
    Store(usize),
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
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
    Jump(usize),
    JumpIfFalse(usize),
    Loop(usize),
    Call(u8),
    Return,
    // Property operations
    GetProperty(usize),  // property name constant index
    SetProperty(usize),  // property name constant index
    // Collection operations
    MakeList(usize),     // element count
    MakeDict(usize),     // pair count
    // Variable operations
    LoadGlobal(usize),   // variable name constant index
    StoreGlobal(usize),  // variable name constant index
    // Function operations
    DefineFunction(usize), // function value constant index
    CallFunction(u8),      // argument count
}

#[derive(Debug, Clone)]
pub struct Bytecode {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
}

impl Bytecode {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn emit(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }
}

impl Default for Bytecode {
    fn default() -> Self {
        Self::new()
    }
}