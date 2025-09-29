mod helpers;

use helpers::*;
use lugli_common::Value;

#[test]
fn arithmetic_operators() {
	assert_execution_succeeds("10 + 5", Value::Number(15.0));
	assert_execution_succeeds("10 - 5", Value::Number(5.0));
	assert_execution_succeeds("10 * 5", Value::Number(50.0));
	assert_execution_succeeds("10 / 5", Value::Number(2.0));
	assert_execution_succeeds("10 % 3", Value::Number(1.0));
}

#[test]
fn comparison_operators() {
	assert_execution_succeeds("10 > 5", Value::Bool(true));
	assert_execution_succeeds("10 >= 10", Value::Bool(true));
	assert_execution_succeeds("5 < 10", Value::Bool(true));
	assert_execution_succeeds("5 <= 5", Value::Bool(true));
	assert_execution_succeeds("5 == 5", Value::Bool(true));
	assert_execution_succeeds("5 != 3", Value::Bool(true));
}

#[test]
fn logical_operators() {
	assert_execution_succeeds("true && true", Value::Bool(true));
	assert_execution_succeeds("true && false", Value::Bool(false));
	assert_execution_succeeds("true || false", Value::Bool(true));
	assert_execution_succeeds("false || false", Value::Bool(false));
}

#[test]
fn unary_operators() {
	assert_execution_succeeds("-42", Value::Number(-42.0));
	assert_execution_succeeds("!true", Value::Bool(false));
	assert_execution_succeeds("!false", Value::Bool(true));
}

#[test]
fn complex_expressions() {
	assert_execution_succeeds("(10 + 5) * 2", Value::Number(30.0));
	assert_execution_succeeds("10 > 5 && 3 < 8", Value::Bool(true));
	assert_execution_succeeds("10 >= 10 || 5 != 5", Value::Bool(true));
}
