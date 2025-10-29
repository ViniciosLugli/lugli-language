// Test helper functions for VM integration tests
//
// Provides utilities for testing the complete Lugli execution pipeline:
// - Parsing source code
// - Compiling to bytecode
// - Running in the VM
// - Asserting on results
//
// Used by most test files in crates/lugli-vm/tests/

use lugli_common::Value;
use lugli_parser::{Parser, parse};
use lugli_vm::{compile, compile_and_run, run};

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
                    },
                }
            }
            Err(e) => TestResult {
                success: false,
                value: None,
                error: Some(format!("Compile error: {}", e)),
                instruction_count: 0,
                constant_count: 0,
            },
        },
        Err(e) => TestResult {
            success: false,
            value: None,
            error: Some(format!("Parse error: {}", e)),
            instruction_count: 0,
            constant_count: 0,
        },
    }
}

pub fn assert_execution_succeeds(source: &str, expected_value: Value) {
    let result = test_lugli_pipeline(source);
    assert!(result.success);
    assert!(result.value.unwrap().equals(&expected_value));
}

pub fn run_test(source: &str) {
    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse().unwrap();
    compile_and_run(&ast).unwrap();
}

pub fn run_test_expect_error(source: &str) -> bool {
    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse().unwrap();
    compile_and_run(&ast).is_err()
}

pub fn assert_value_eq(actual: &Value, expected: &Value) {
    assert!(actual.equals(expected), "Values not equal: {:?} != {:?}", actual, expected);
}
