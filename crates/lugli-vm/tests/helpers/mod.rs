// Test helper functions for VM integration tests

#![allow(dead_code)] // Helpers used selectively across different test files

use lugli_common::Value;
use lugli_parser::Parser;
use lugli_vm::{Bytecode, compile, compile_and_run, run};

pub fn run_test(source: &str) {
    let mut parser = Parser::new(source).unwrap();
    let (ast, span_map) = parser.parse().unwrap();
    compile_and_run(&ast, span_map).unwrap();
}

pub fn run_test_expect_error(source: &str) -> bool {
    let mut parser = Parser::new(source).unwrap();
    let (ast, span_map) = parser.parse().unwrap();
    compile_and_run(&ast, span_map).is_err()
}

pub fn assert_value_eq(actual: &Value, expected: &Value) {
    assert!(actual.equals(expected), "Values not equal: {:?} != {:?}", actual, expected);
}

pub fn assert_execution_succeeds(source: &str, expected_value: Value) {
    let mut parser = Parser::new(source).unwrap();
    let (ast, span_map) = parser.parse().unwrap();
    let result = compile_and_run(&ast, span_map).unwrap();
    assert!(result.equals(&expected_value));
}

pub fn compile_source(source: &str) -> (Bytecode, Value) {
    let mut parser = Parser::new(source).unwrap();
    let (ast, span_map) = parser.parse().unwrap();
    let bytecode = compile(&ast, span_map).unwrap();
    let result = run(&bytecode).unwrap();
    (bytecode, result)
}
