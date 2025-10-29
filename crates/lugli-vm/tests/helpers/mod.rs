// Test helper functions for VM integration tests

use lugli_common::Value;
use lugli_parser::Parser;
use lugli_vm::compile_and_run;

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

pub fn assert_execution_succeeds(source: &str, expected_value: Value) {
    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse().unwrap();
    let result = compile_and_run(&ast).unwrap();
    assert!(result.equals(&expected_value));
}
