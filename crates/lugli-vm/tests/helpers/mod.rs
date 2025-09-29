use lugli_vm::{compile, run};
use lugli_parser::parse;
use lugli_common::Value;

#[derive(Debug)]
pub struct TestResult {
    pub success: bool,
    pub value: Option<Value>,
    #[allow(dead_code)]
    pub error: Option<String>,
    #[allow(dead_code)]
    pub instruction_count: usize,
    #[allow(dead_code)]
    pub constant_count: usize,
}

pub fn test_lugli_pipeline(source: &str) -> TestResult {
    let source_clean = source.trim();

    match parse(source_clean) {
        Ok(program) => match compile(&program) {
            Ok(bytecode) => {
                let instruction_count = bytecode.instructions.len();
                let constant_count = bytecode.constants.len();

                match run(&bytecode) {
                    Ok(value) => TestResult {
                        success: true,
                        value: Some(value),
                        error: None,
                        instruction_count,
                        constant_count,
                    },
                    Err(e) => TestResult {
                        success: false,
                        value: None,
                        error: Some(format!("Runtime error: {}", e)),
                        instruction_count,
                        constant_count,
                    }
                }
            }
            Err(e) => TestResult {
                success: false,
                value: None,
                error: Some(format!("Compile error: {}", e)),
                instruction_count: 0,
                constant_count: 0,
            }
        }
        Err(e) => TestResult {
            success: false,
            value: None,
            error: Some(format!("Parse error: {}", e)),
            instruction_count: 0,
            constant_count: 0,
        }
    }
}

pub fn assert_execution_succeeds(source: &str, expected_value: Value) {
    let result = test_lugli_pipeline(source);
    assert!(result.success);
    assert!(result.value.unwrap().equals(&expected_value));
}

#[allow(dead_code)]
pub fn assert_compiles_to_instructions(source: &str, expected_count: usize) {
    let result = test_lugli_pipeline(source);
    assert!(result.success);
    assert_eq!(result.instruction_count, expected_count);
}

#[allow(dead_code)]
pub fn assert_compiles_to_constants(source: &str, expected_count: usize) {
    let result = test_lugli_pipeline(source);
    assert!(result.success);
    assert_eq!(result.constant_count, expected_count);
}