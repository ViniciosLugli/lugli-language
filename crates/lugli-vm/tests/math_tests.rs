use lugli_common::Value;
use lugli_parser::Parser;
use lugli_vm::compile_and_run;

fn run_and_get_value(source: &str) -> Value {
    let mut parser = Parser::new(source).unwrap();
    let (ast, span_map) = parser.parse().unwrap();
    compile_and_run(&ast, span_map).unwrap()
}

#[test]
fn test_abs_positive() {
    let source = "abs(42)";
    let result = run_and_get_value(source);
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_abs_negative() {
    let source = "abs(-42)";
    let result = run_and_get_value(source);
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_abs_zero() {
    let source = "abs(0)";
    let result = run_and_get_value(source);
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_abs_decimal() {
    let source = "abs(-3.14)";
    let result = run_and_get_value(source);
    assert_eq!(result, Value::Number(3.14));
}

#[test]
fn test_round_no_places() {
    let source = "round(3.7)";
    let result = run_and_get_value(source);
    assert_eq!(result, Value::Number(4.0));
}

#[test]
fn test_round_down() {
    let source = "round(3.4)";
    let result = run_and_get_value(source);
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_round_with_places() {
    let source = "round(3.14159, 2)";
    let result = run_and_get_value(source);
    assert_eq!(result, Value::Number(3.14));
}

#[test]
fn test_round_with_places_three() {
    let source = "round(3.14159, 3)";
    let result = run_and_get_value(source);
    assert_eq!(result, Value::Number(3.142));
}

#[test]
fn test_round_negative() {
    let source = "round(-3.7)";
    let result = run_and_get_value(source);
    assert_eq!(result, Value::Number(-4.0));
}

#[test]
fn test_pow_basic() {
    let source = "pow(2, 3)";
    let result = run_and_get_value(source);
    assert_eq!(result, Value::Number(8.0));
}

#[test]
fn test_pow_zero_exponent() {
    let source = "pow(5, 0)";
    let result = run_and_get_value(source);
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_pow_negative_exponent() {
    let source = "pow(2, -2)";
    let result = run_and_get_value(source);
    assert_eq!(result, Value::Number(0.25));
}

#[test]
fn test_pow_decimal_base() {
    let source = "pow(2.5, 2)";
    let result = run_and_get_value(source);
    assert_eq!(result, Value::Number(6.25));
}

#[test]
fn test_pow_decimal_exponent() {
    let source = "pow(4, 0.5)";
    let result = run_and_get_value(source);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_math_functions_combined() {
    let source = r#"
        let x = -5.7
        let y = abs(x)  # 5.7
        let z = round(y)  # 6.0
        let w = pow(z, 2)  # 36.0
        w
    "#;
    let result = run_and_get_value(source);
    assert_eq!(result, Value::Number(36.0));
}

#[test]
fn test_abs_in_expression() {
    let source = "abs(-10) + abs(-20)";
    let result = run_and_get_value(source);
    assert_eq!(result, Value::Number(30.0));
}

#[test]
fn test_round_in_expression() {
    let source = "round(1.4) + round(2.6)";
    let result = run_and_get_value(source);
    assert_eq!(result, Value::Number(4.0));
}

#[test]
fn test_pow_in_expression() {
    let source = "pow(2, 2) + pow(3, 2)";
    let result = run_and_get_value(source);
    assert_eq!(result, Value::Number(13.0));
}
