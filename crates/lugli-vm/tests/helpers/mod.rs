// Test helper functions for VM integration tests
#![allow(dead_code)]

use lugli_common::Value;
use lugli_parser::Parser;
use lugli_vm::{Bytecode, Vm};

pub fn run_test(source: &str) {
    let mut parser = Parser::new(source).unwrap();
    let (ast, span_map) = parser.parse().unwrap();
    let mut vm = Vm::new();
    vm.compile_and_run(&ast, span_map).unwrap();
}

pub fn run_test_expect_error(source: &str) -> bool {
    let mut parser = Parser::new(source).unwrap();
    let (ast, span_map) = parser.parse().unwrap();
    let mut vm = Vm::new();
    vm.compile_and_run(&ast, span_map).is_err()
}

pub fn assert_value_eq(actual: &Value, expected: &Value) {
    assert!(actual.equals(expected), "Values not equal: {:?} != {:?}", actual, expected);
}

pub fn assert_execution_succeeds(source: &str, expected_value: Value) {
    let mut parser = Parser::new(source).unwrap();
    let (ast, span_map) = parser.parse().unwrap();
    let mut vm = Vm::new();
    let result = vm.compile_and_run(&ast, span_map).unwrap();
    assert!(result.equals(&expected_value));
}

pub fn compile_source(source: &str) -> (Bytecode, Value) {
    let mut parser = Parser::new(source).unwrap();
    let (ast, span_map) = parser.parse().unwrap();
    let mut vm = Vm::new();
    let bytecode = vm.compile(&ast, span_map).unwrap();
    let result = vm.run(&bytecode).unwrap();
    (bytecode, result)
}
