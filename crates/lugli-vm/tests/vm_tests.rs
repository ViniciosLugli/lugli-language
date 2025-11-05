mod helpers;

use helpers::assert_value_eq;
use lugli_common::{LugliError, StringPool, Value};
use lugli_vm::{Bytecode, Instruction, Machine};

// Simple len function for testing
fn test_len_callback(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    if let Some(arg) = args.first() {
        match arg {
            Value::String(s) => {
                let text = pool.resolve(*s);
                Ok(Value::Number(text.len() as f64))
            }
            _ => Err(LugliError::runtime("len() requires string argument")),
        }
    } else {
        Err(LugliError::runtime("len() missing required argument"))
    }
}

fn run_vm(instructions: Vec<Instruction>, constants: Vec<Value>) -> Result<(Value, Bytecode), LugliError> {
    let mut vm = Machine::new();
    let mut bytecode = Bytecode::new();

    bytecode.instructions = instructions;
    bytecode.constants = constants;

    let result = vm.run(&bytecode)?;
    Ok((result, bytecode))
}

#[test]
fn test_vm_constant() {
    let (result, _) = run_vm(vec![Instruction::Constant(0), Instruction::Return], vec![Value::Number(42.0)]).unwrap();

    assert_value_eq(&result, &Value::Number(42.0));
}

#[test]
fn test_vm_arithmetic_add() {
    let (result, _) = run_vm(
        vec![
            Instruction::Constant(0), // 10
            Instruction::Constant(1), // 20
            Instruction::Add,
            Instruction::Return,
        ],
        vec![Value::Number(10.0), Value::Number(20.0)],
    )
    .unwrap();

    assert_value_eq(&result, &Value::Number(30.0));
}

#[test]
fn test_vm_arithmetic_subtract() {
    let (result, _) = run_vm(
        vec![
            Instruction::Constant(0), // 50
            Instruction::Constant(1), // 20
            Instruction::Subtract,
            Instruction::Return,
        ],
        vec![Value::Number(50.0), Value::Number(20.0)],
    )
    .unwrap();

    assert_value_eq(&result, &Value::Number(30.0));
}

#[test]
fn test_vm_arithmetic_multiply() {
    let (result, _) = run_vm(
        vec![
            Instruction::Constant(0), // 6
            Instruction::Constant(1), // 7
            Instruction::Multiply,
            Instruction::Return,
        ],
        vec![Value::Number(6.0), Value::Number(7.0)],
    )
    .unwrap();

    assert_value_eq(&result, &Value::Number(42.0));
}

#[test]
fn test_vm_arithmetic_divide() {
    let (result, _) = run_vm(
        vec![
            Instruction::Constant(0), // 84
            Instruction::Constant(1), // 2
            Instruction::Divide,
            Instruction::Return,
        ],
        vec![Value::Number(84.0), Value::Number(2.0)],
    )
    .unwrap();

    assert_value_eq(&result, &Value::Number(42.0));
}

#[test]
fn test_vm_negate() {
    let (result, _) = run_vm(
        vec![
            Instruction::Constant(0), // 42
            Instruction::Negate,
            Instruction::Return,
        ],
        vec![Value::Number(42.0)],
    )
    .unwrap();

    assert_value_eq(&result, &Value::Number(-42.0));
}

#[test]
fn test_vm_not() {
    let (result, _) = run_vm(
        vec![
            Instruction::Constant(0), // true
            Instruction::Not,
            Instruction::Return,
        ],
        vec![Value::Bool(true)],
    )
    .unwrap();

    assert_value_eq(&result, &Value::Bool(false));
}

#[test]
fn test_vm_comparison_equal() {
    let (result, _) = run_vm(
        vec![
            Instruction::Constant(0), // 42
            Instruction::Constant(1), // 42
            Instruction::Equal,
            Instruction::Return,
        ],
        vec![Value::Number(42.0), Value::Number(42.0)],
    )
    .unwrap();

    assert_value_eq(&result, &Value::Bool(true));
}

#[test]
fn test_vm_comparison_greater() {
    let (result, _) = run_vm(
        vec![
            Instruction::Constant(0), // 50
            Instruction::Constant(1), // 30
            Instruction::Greater,
            Instruction::Return,
        ],
        vec![Value::Number(50.0), Value::Number(30.0)],
    )
    .unwrap();

    assert_value_eq(&result, &Value::Bool(true));
}

#[test]
fn test_vm_comparison_less() {
    let (result, _) = run_vm(
        vec![
            Instruction::Constant(0), // 30
            Instruction::Constant(1), // 50
            Instruction::Less,
            Instruction::Return,
        ],
        vec![Value::Number(30.0), Value::Number(50.0)],
    )
    .unwrap();

    assert_value_eq(&result, &Value::Bool(true));
}

#[test]
fn test_vm_variable_load_store() {
    // The Store instruction expects space for locals on the stack.
    // In compiled code, VarDecl pushes the initial value.
    // For this test, we simulate that by just leaving the value on the stack.
    let (result, _) = run_vm(
        vec![
            Instruction::Constant(0), // 42 - this stays on stack as local 0
            Instruction::Load(0),     // load from local 0
            Instruction::Return,
        ],
        vec![Value::Number(42.0)],
    )
    .unwrap();

    assert_value_eq(&result, &Value::Number(42.0));
}

#[test]
fn test_vm_jump() {
    let (result, _) = run_vm(
        vec![
            Instruction::Constant(0), // 1 - this gets pushed
            Instruction::Jump(4),     // jump to instruction 4
            Instruction::Constant(1), // 2 - this is skipped
            Instruction::Add,         // 3 - this is skipped
            Instruction::Return,      // 4 - jump here
        ],
        vec![Value::Number(1.0), Value::Number(2.0)],
    )
    .unwrap();

    assert_value_eq(&result, &Value::Number(1.0)); // Should return 1, not 3
}

#[test]
fn test_vm_jump_if_false() {
    let (result, _) = run_vm(
        vec![
            Instruction::Constant(0),    // false
            Instruction::JumpIfFalse(4), // should jump to instruction 4
            Instruction::Constant(1),    // 1 - this is skipped
            Instruction::Return,         // this is skipped
            Instruction::Constant(2),    // 2 - jump here
            Instruction::Return,
        ],
        vec![Value::Bool(false), Value::Number(1.0), Value::Number(2.0)],
    )
    .unwrap();

    assert_value_eq(&result, &Value::Number(2.0));
}

#[test]
fn test_vm_complex_expression() {
    // (10 + 20) * 2 = 60
    let (result, _) = run_vm(
        vec![
            Instruction::Constant(0), // 10
            Instruction::Constant(1), // 20
            Instruction::Add,         // 30
            Instruction::Constant(2), // 2
            Instruction::Multiply,    // 60
            Instruction::Return,
        ],
        vec![Value::Number(10.0), Value::Number(20.0), Value::Number(2.0)],
    )
    .unwrap();

    assert_value_eq(&result, &Value::Number(60.0));
}

#[test]
fn test_vm_native_function_call() {
    let mut bytecode = Bytecode::new();
    let hello_id = bytecode.string_pool.borrow_mut().intern("hello");

    bytecode.instructions = vec![
        Instruction::Constant(1), // "hello" string (argument)
        Instruction::Constant(0), // "len" function
        Instruction::Call(1),     // call len("hello")
        Instruction::Return,
    ];
    bytecode.constants = vec![
        Value::NativeFunction {
            name: "len".to_string(),
            callback: test_len_callback,
            arity: 1,
        },
        Value::String(hello_id),
    ];

    let mut vm = Machine::new();
    let result = vm.run(&bytecode).unwrap();

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

// NOTE: String concatenation test removed because it requires going through
// the full compiler pipeline to get proper string pool handling.
// String concatenation is tested in pipeline_tests.rs instead.

#[test]
fn test_vm_pop_operation() {
    let (result, _) = run_vm(
        vec![
            Instruction::Constant(0), // 1
            Instruction::Constant(1), // 2
            Instruction::Pop,         // pop 2, stack now has [1]
            Instruction::Return,      // return 1
        ],
        vec![Value::Number(1.0), Value::Number(2.0)],
    )
    .unwrap();

    assert_value_eq(&result, &Value::Number(1.0));
}
#[test]
fn test_to_string_nan() {
    let (result, bytecode) =
        run_vm(vec![Instruction::Constant(0), Instruction::ToString, Instruction::Return], vec![Value::Number(f64::NAN)]).unwrap();

    if let Value::String(result_id) = result {
        let pool = bytecode.string_pool.borrow();
        let resolved = pool.resolve(result_id);
        assert_eq!(resolved, "NaN");
    } else {
        panic!("Expected String result, got {:?}", result);
    }
}

#[test]
fn test_to_string_infinity() {
    let (result, bytecode) =
        run_vm(vec![Instruction::Constant(0), Instruction::ToString, Instruction::Return], vec![Value::Number(f64::INFINITY)]).unwrap();

    if let Value::String(result_id) = result {
        let pool = bytecode.string_pool.borrow();
        let resolved = pool.resolve(result_id);
        assert_eq!(resolved, "Infinity");
    } else {
        panic!("Expected String result, got {:?}", result);
    }
}

#[test]
fn test_to_string_neg_infinity() {
    let (result, bytecode) =
        run_vm(vec![Instruction::Constant(0), Instruction::ToString, Instruction::Return], vec![Value::Number(f64::NEG_INFINITY)]).unwrap();

    if let Value::String(result_id) = result {
        let pool = bytecode.string_pool.borrow();
        let resolved = pool.resolve(result_id);
        assert_eq!(resolved, "-Infinity");
    } else {
        panic!("Expected String result, got {:?}", result);
    }
}

#[test]
fn test_to_string_whole_number() {
    let (result, bytecode) = run_vm(vec![Instruction::Constant(0), Instruction::ToString, Instruction::Return], vec![Value::Number(42.0)]).unwrap();

    if let Value::String(result_id) = result {
        let pool = bytecode.string_pool.borrow();
        let resolved = pool.resolve(result_id);
        assert_eq!(resolved, "42");
    } else {
        panic!("Expected String result, got {:?}", result);
    }
}

#[test]
fn test_to_string_decimal() {
    let (result, bytecode) = run_vm(vec![Instruction::Constant(0), Instruction::ToString, Instruction::Return], vec![Value::Number(2.5)]).unwrap();

    if let Value::String(result_id) = result {
        let pool = bytecode.string_pool.borrow();
        let resolved = pool.resolve(result_id);
        assert_eq!(resolved, "2.5");
    } else {
        panic!("Expected String result, got {:?}", result);
    }
}
