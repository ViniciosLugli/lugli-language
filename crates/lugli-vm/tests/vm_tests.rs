use lugli_common::{Value, LugliError};

// Simple len function for testing
fn test_len_callback(args: &[Value]) -> Result<Value, LugliError> {
    if let Some(arg) = args.first() {
        match arg {
            Value::String(s) => Ok(Value::Number(s.len() as f64)),
            _ => Err(LugliError::runtime("len() requires string argument"))
        }
    } else {
        Err(LugliError::runtime("len() missing required argument"))
    }
}

// Helper function for value equality testing
fn assert_value_eq(actual: &Value, expected: &Value) {
    assert!(actual.equals(expected), "Values not equal: {:?} != {:?}", actual, expected);
}
use lugli_vm::{VirtualMachine, Bytecode, Instruction};

fn run_vm(instructions: Vec<Instruction>, constants: Vec<Value>) -> Result<Value, LugliError> {
    let mut vm = VirtualMachine::new();
    let mut bytecode = Bytecode::new();

    bytecode.instructions = instructions;
    bytecode.constants = constants;

    vm.run(&bytecode)
}

#[test]
fn test_vm_constant() {
    let result = run_vm(
        vec![
            Instruction::Constant(0),
            Instruction::Return,
        ],
        vec![Value::Number(42.0)],
    ).unwrap();

    assert_value_eq(&result, &Value::Number(42.0));
}

#[test]
fn test_vm_arithmetic_add() {
    let result = run_vm(
        vec![
            Instruction::Constant(0), // 10
            Instruction::Constant(1), // 20
            Instruction::Add,
            Instruction::Return,
        ],
        vec![Value::Number(10.0), Value::Number(20.0)],
    ).unwrap();

    assert_value_eq(&result, &Value::Number(30.0));
}

#[test]
fn test_vm_arithmetic_subtract() {
    let result = run_vm(
        vec![
            Instruction::Constant(0), // 50
            Instruction::Constant(1), // 20
            Instruction::Subtract,
            Instruction::Return,
        ],
        vec![Value::Number(50.0), Value::Number(20.0)],
    ).unwrap();

    assert_value_eq(&result, &Value::Number(30.0));
}

#[test]
fn test_vm_arithmetic_multiply() {
    let result = run_vm(
        vec![
            Instruction::Constant(0), // 6
            Instruction::Constant(1), // 7
            Instruction::Multiply,
            Instruction::Return,
        ],
        vec![Value::Number(6.0), Value::Number(7.0)],
    ).unwrap();

    assert_value_eq(&result, &Value::Number(42.0));
}

#[test]
fn test_vm_arithmetic_divide() {
    let result = run_vm(
        vec![
            Instruction::Constant(0), // 84
            Instruction::Constant(1), // 2
            Instruction::Divide,
            Instruction::Return,
        ],
        vec![Value::Number(84.0), Value::Number(2.0)],
    ).unwrap();

    assert_value_eq(&result, &Value::Number(42.0));
}

#[test]
fn test_vm_negate() {
    let result = run_vm(
        vec![
            Instruction::Constant(0), // 42
            Instruction::Negate,
            Instruction::Return,
        ],
        vec![Value::Number(42.0)],
    ).unwrap();

    assert_value_eq(&result, &Value::Number(-42.0));
}

#[test]
fn test_vm_not() {
    let result = run_vm(
        vec![
            Instruction::Constant(0), // true
            Instruction::Not,
            Instruction::Return,
        ],
        vec![Value::Bool(true)],
    ).unwrap();

    assert_value_eq(&result, &Value::Bool(false));
}

#[test]
fn test_vm_comparison_equal() {
    let result = run_vm(
        vec![
            Instruction::Constant(0), // 42
            Instruction::Constant(1), // 42
            Instruction::Equal,
            Instruction::Return,
        ],
        vec![Value::Number(42.0), Value::Number(42.0)],
    ).unwrap();

    assert_value_eq(&result, &Value::Bool(true));
}

#[test]
fn test_vm_comparison_greater() {
    let result = run_vm(
        vec![
            Instruction::Constant(0), // 50
            Instruction::Constant(1), // 30
            Instruction::Greater,
            Instruction::Return,
        ],
        vec![Value::Number(50.0), Value::Number(30.0)],
    ).unwrap();

    assert_value_eq(&result, &Value::Bool(true));
}

#[test]
fn test_vm_comparison_less() {
    let result = run_vm(
        vec![
            Instruction::Constant(0), // 30
            Instruction::Constant(1), // 50
            Instruction::Less,
            Instruction::Return,
        ],
        vec![Value::Number(30.0), Value::Number(50.0)],
    ).unwrap();

    assert_value_eq(&result, &Value::Bool(true));
}

#[test]
fn test_vm_variable_load_store() {
    let result = run_vm(
        vec![
            Instruction::Constant(0), // 42
            Instruction::Store(0),     // store in local 0
            Instruction::Load(0),      // load from local 0
            Instruction::Return,
        ],
        vec![Value::Number(42.0)],
    ).unwrap();

    assert_value_eq(&result, &Value::Number(42.0));
}

#[test]
fn test_vm_jump() {
    let result = run_vm(
        vec![
            Instruction::Constant(0), // 1 - this gets pushed
            Instruction::Jump(4),     // jump to instruction 4
            Instruction::Constant(1), // 2 - this is skipped
            Instruction::Add,         // 3 - this is skipped
            Instruction::Return,      // 4 - jump here
        ],
        vec![Value::Number(1.0), Value::Number(2.0)],
    ).unwrap();

    assert_value_eq(&result, &Value::Number(1.0)); // Should return 1, not 3
}

#[test]
fn test_vm_jump_if_false() {
    let result = run_vm(
        vec![
            Instruction::Constant(0),     // false
            Instruction::JumpIfFalse(4),  // should jump to instruction 4
            Instruction::Constant(1),     // 1 - this is skipped
            Instruction::Return,          // this is skipped
            Instruction::Constant(2),     // 2 - jump here
            Instruction::Return,
        ],
        vec![Value::Bool(false), Value::Number(1.0), Value::Number(2.0)],
    ).unwrap();

    assert_value_eq(&result, &Value::Number(2.0));
}

#[test]
fn test_vm_complex_expression() {
    // (10 + 20) * 2 = 60
    let result = run_vm(
        vec![
            Instruction::Constant(0), // 10
            Instruction::Constant(1), // 20
            Instruction::Add,         // 30
            Instruction::Constant(2), // 2
            Instruction::Multiply,    // 60
            Instruction::Return,
        ],
        vec![Value::Number(10.0), Value::Number(20.0), Value::Number(2.0)],
    ).unwrap();

    assert_value_eq(&result, &Value::Number(60.0));
}

#[test]
fn test_vm_native_function_call() {
    let result = run_vm(
        vec![
            Instruction::Constant(0), // "len" function
            Instruction::Constant(1), // "hello" string
            Instruction::Call(1),     // call len("hello")
            Instruction::Return,
        ],
        vec![
            Value::NativeFunction {
                name: "len".to_string(),
                callback: test_len_callback,
                arity: 1
            },
            Value::String("hello".to_string()),
        ],
    ).unwrap();

    assert_value_eq(&result, &Value::Number(5.0));
}

#[test]
fn test_vm_stack_underflow() {
    let result = run_vm(
        vec![
            Instruction::Add, // try to add without operands
            Instruction::Return,
        ],
        vec![],
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Stack underflow"));
}

#[test]
fn test_vm_string_concatenation() {
    let result = run_vm(
        vec![
            Instruction::Constant(0), // "Hello"
            Instruction::Constant(1), // " World"
            Instruction::Add,         // "Hello World"
            Instruction::Return,
        ],
        vec![Value::String("Hello".to_string()), Value::String(" World".to_string())],
    ).unwrap();

    assert_value_eq(&result, &Value::String("Hello World".to_string()));
}

#[test]
fn test_vm_pop_operation() {
    let result = run_vm(
        vec![
            Instruction::Constant(0), // 1
            Instruction::Constant(1), // 2
            Instruction::Pop,         // pop 2, stack now has [1]
            Instruction::Return,      // return 1
        ],
        vec![Value::Number(1.0), Value::Number(2.0)],
    ).unwrap();

    assert_value_eq(&result, &Value::Number(1.0));
}