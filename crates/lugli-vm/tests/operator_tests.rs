// Operator tests - validates all language operators
//
// Tests arithmetic, comparison, logical, and unary operators including:
// - Basic arithmetic: +, -, *, /, %
// - Comparisons: >, >=, <, <=, ==, !=
// - Logical: &&, ||
// - Unary: -, !
// - String and list concatenation: +
// - Complex expressions with multiple operators

mod helpers;

use helpers::assert_execution_succeeds;
use lugli_common::Value;

#[test]
fn test_arithmetic_operators() {
    assert_execution_succeeds("10 + 5", Value::Number(15.0));
    assert_execution_succeeds("10 - 5", Value::Number(5.0));
    assert_execution_succeeds("10 * 5", Value::Number(50.0));
    assert_execution_succeeds("10 / 5", Value::Number(2.0));
    assert_execution_succeeds("10 % 3", Value::Number(1.0));
}

#[test]
fn test_comparison_operators() {
    assert_execution_succeeds("10 > 5", Value::Bool(true));
    assert_execution_succeeds("10 >= 10", Value::Bool(true));
    assert_execution_succeeds("5 < 10", Value::Bool(true));
    assert_execution_succeeds("5 <= 5", Value::Bool(true));
    assert_execution_succeeds("5 == 5", Value::Bool(true));
    assert_execution_succeeds("5 != 3", Value::Bool(true));
}

#[test]
fn test_logical_operators() {
    assert_execution_succeeds("true && true", Value::Bool(true));
    assert_execution_succeeds("true && false", Value::Bool(false));
    assert_execution_succeeds("true || false", Value::Bool(true));
    assert_execution_succeeds("false || false", Value::Bool(false));
}

#[test]
fn test_unary_operators() {
    assert_execution_succeeds("-42", Value::Number(-42.0));
    assert_execution_succeeds("!true", Value::Bool(false));
    assert_execution_succeeds("!false", Value::Bool(true));
}

#[test]
fn test_string_concatenation() {
    assert_execution_succeeds(r#""hello" + " " + "world""#, Value::String("hello world".to_string()));
    assert_execution_succeeds(r#""a" + "b""#, Value::String("ab".to_string()));
}

#[test]
fn test_list_concatenation() {
    use helpers::run_test;

    // Basic concatenation
    let source = r#"
        let result = [1] + [2, 3]
        if result.len() != 3 { return "FAIL: length" }
        if result[0] != 1 { return "FAIL: first" }
        if result[1] != 2 { return "FAIL: second" }
        if result[2] != 3 { return "FAIL: third" }
    "#;
    run_test(source);

    // Concatenating with empty list
    let source = r#"
        let result = [1, 2, 3] + []
        if result.len() != 3 { return "FAIL" }
    "#;
    run_test(source);

    // Multiple concatenations
    let source = r#"
        let result = [1, 2] + [3, 4] + [5, 6]
        if result.len() != 6 { return "FAIL: length" }
        if result[0] != 1 { return "FAIL" }
        if result[5] != 6 { return "FAIL" }
    "#;
    run_test(source);

    // Quicksort-style usage
    let source = r#"
        let less = [1, 2]
        let equal = [3]
        let greater = [4, 5]
        let result = less + equal + greater
        if result.len() != 5 { return "FAIL" }
    "#;
    run_test(source);
}

#[test]
fn test_dict_numeric_keys() {
    use helpers::run_test;

    // Numeric keys - set and get
    let source = r#"
        let dict = {}
        dict[0] = "zero"
        dict[1] = "one"
        dict[42] = "answer"

        if dict[0] != "zero" { return "FAIL: get 0" }
        if dict[1] != "one" { return "FAIL: get 1" }
        if dict[42] != "answer" { return "FAIL: get 42" }
    "#;
    run_test(source);

    // Mixed numeric and string keys
    let source = r#"
        let dict = {}
        dict[0] = "numeric"
        dict["key"] = "string"

        if dict[0] != "numeric" { return "FAIL: numeric" }
        if dict["key"] != "string" { return "FAIL: string" }
    "#;
    run_test(source);

    // Float keys
    let source = r#"
        let dict = {}
        dict[3.14] = "pi"
        if dict[3.14] != "pi" { return "FAIL" }
    "#;
    run_test(source);

    // Loop with numeric keys
    let source = r#"
        let dict = {}
        mut i = 0
        while i < 10 {
            dict[i] = i * 2
            i = i + 1
        }

        if dict[0] != 0 { return "FAIL: 0" }
        if dict[5] != 10 { return "FAIL: 5" }
        if dict[9] != 18 { return "FAIL: 9" }
    "#;
    run_test(source);
}

#[test]
fn test_complex_expressions() {
    assert_execution_succeeds("(10 + 5) * 2", Value::Number(30.0));
    assert_execution_succeeds("10 > 5 && 3 < 8", Value::Bool(true));
    assert_execution_succeeds("10 >= 10 || 5 != 5", Value::Bool(true));
}
