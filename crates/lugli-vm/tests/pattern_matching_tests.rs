use lugli_common::Value;
use lugli_parser::Parser;
use lugli_vm::{Bytecode, compile, run};

fn run_and_get_value(source: &str) -> (Bytecode, Value) {
    let mut parser = Parser::new(source).unwrap();
    let (ast, span_map) = parser.parse().unwrap();
    let bytecode = compile(&ast, span_map).unwrap();
    let result = run(&bytecode).unwrap();
    (bytecode, result)
}

#[test]
fn test_literal_pattern_number() {
    let source = r#"
        let x = 42
        match x {
            0 => "zero",
            42 => "answer",
            100 => "hundred",
            _ => "other"
        }
    "#;

    let (bytecode, result) = run_and_get_value(source);
    let mut pool = bytecode.string_pool.borrow_mut();
    let answer_id = pool.intern("answer");
    assert_eq!(result, Value::String(answer_id));
}

#[test]
fn test_literal_pattern_string() {
    let source = r#"
        let status = "success"
        match status {
            "success" => 1,
            "error" => 0,
            _ => -1
        }
    "#;

    let (_, result) = run_and_get_value(source);
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_literal_pattern_boolean() {
    let source = r#"
        let flag = true
        match flag {
            true => "yes",
            false => "no"
        }
    "#;

    let (bytecode, result) = run_and_get_value(source);
    let mut pool = bytecode.string_pool.borrow_mut();
    let yes_id = pool.intern("yes");
    assert_eq!(result, Value::String(yes_id));
}

#[test]
fn test_literal_pattern_null() {
    let source = r#"
        let value = null
        match value {
            null => "nothing",
            _ => "something"
        }
    "#;

    let (bytecode, result) = run_and_get_value(source);
    let mut pool = bytecode.string_pool.borrow_mut();
    let nothing_id = pool.intern("nothing");
    assert_eq!(result, Value::String(nothing_id));
}

#[test]
fn test_wildcard_pattern() {
    let source = r#"
        let x = 999
        match x {
            0 => "zero",
            1 => "one",
            _ => "many"
        }
    "#;

    let (bytecode, result) = run_and_get_value(source);
    let mut pool = bytecode.string_pool.borrow_mut();
    let many_id = pool.intern("many");
    assert_eq!(result, Value::String(many_id));
}

#[test]
fn test_identifier_pattern_binding() {
    let source = r#"
        let x = 42
        match x {
            0 => "zero",
            n => n * 2
        }
    "#;

    let (_, result) = run_and_get_value(source);
    assert_eq!(result, Value::Number(84.0));
}

#[test]
fn test_guard_with_literal_pattern() {
    let source = r#"
        let x = 42
        match x {
            42 if x > 50 => "big answer",
            42 if x > 0 => "small answer",
            42 => "exact answer",
            _ => "other"
        }
    "#;

    let (bytecode, result) = run_and_get_value(source);
    let mut pool = bytecode.string_pool.borrow_mut();
    let small_answer_id = pool.intern("small answer");
    assert_eq!(result, Value::String(small_answer_id));
}

#[test]
fn test_guard_with_identifier_pattern() {
    let source = r#"
        let x = 15
        match x {
            n if n < 10 => "small",
            n if n < 20 => "medium",
            n => "large"
        }
    "#;

    let (bytecode, result) = run_and_get_value(source);
    let mut pool = bytecode.string_pool.borrow_mut();
    let medium_id = pool.intern("medium");
    assert_eq!(result, Value::String(medium_id));
}

#[test]
fn test_guard_with_complex_condition() {
    let source = r#"
        let x = 25
        match x {
            n if n % 2 == 0 => "even",
            n if n % 2 == 1 => "odd",
            _ => "unknown"
        }
    "#;

    let (bytecode, result) = run_and_get_value(source);
    let mut pool = bytecode.string_pool.borrow_mut();
    let odd_id = pool.intern("odd");
    assert_eq!(result, Value::String(odd_id));
}

#[test]
fn test_match_returns_different_types() {
    let source = r#"
        let x = 1
        match x {
            0 => "zero",
            1 => 100,
            _ => null
        }
    "#;

    let (_, result) = run_and_get_value(source);
    assert_eq!(result, Value::Number(100.0));
}

#[test]
fn test_match_with_expressions() {
    let source = r#"
        let x = 5
        match x + 5 {
            10 => "ten",
            20 => "twenty",
            _ => "other"
        }
    "#;

    let (bytecode, result) = run_and_get_value(source);
    let mut pool = bytecode.string_pool.borrow_mut();
    let ten_id = pool.intern("ten");
    assert_eq!(result, Value::String(ten_id));
}

#[test]
fn test_nested_match() {
    let source = r#"
        let x = 1
        let y = 2
        match x {
            1 => match y {
                2 => "one-two",
                _ => "one-other"
            },
            _ => "other"
        }
    "#;

    let (bytecode, result) = run_and_get_value(source);
    let mut pool = bytecode.string_pool.borrow_mut();
    let one_two_id = pool.intern("one-two");
    assert_eq!(result, Value::String(one_two_id));
}

#[test]
fn test_match_in_variable_assignment() {
    let source = r#"
        let x = 42
        let result = match x {
            0 => "zero",
            42 => "answer",
            _ => "other"
        }
        result
    "#;

    let (bytecode, result) = run_and_get_value(source);
    let mut pool = bytecode.string_pool.borrow_mut();
    let answer_id = pool.intern("answer");
    assert_eq!(result, Value::String(answer_id));
}

// TODO: Uncomment when parser properly handles `?` in method names
// #[test]
// fn test_match_with_method_calls() {
//     let source = r#"
//         let list = [1, 2, 3]
//         match list.len?() {
//             0 => "empty",
//             3 => "three",
//             _ => "other"
//         }
//     "#;
//
//     let result = run_and_get_value(source);
//     assert_eq!(result, Value::String("three".to_string()));
// }

#[test]
fn test_match_first_arm_wins() {
    let source = r#"
        let x = 42
        match x {
            42 => "first",
            42 => "second",
            _ => "other"
        }
    "#;

    let (bytecode, result) = run_and_get_value(source);
    let mut pool = bytecode.string_pool.borrow_mut();
    let first_id = pool.intern("first");
    assert_eq!(result, Value::String(first_id));
}

#[test]
fn test_match_all_patterns_fail_uses_wildcard() {
    let source = r#"
        let x = 999
        match x {
            0 => "zero",
            1 => "one",
            2 => "two",
            _ => "fallback"
        }
    "#;

    let (bytecode, result) = run_and_get_value(source);
    let mut pool = bytecode.string_pool.borrow_mut();
    let fallback_id = pool.intern("fallback");
    assert_eq!(result, Value::String(fallback_id));
}

#[test]
fn test_match_with_arithmetic_in_body() {
    let source = r#"
        let x = 2
        match x {
            1 => 10 + 5,
            2 => 20 * 3,
            _ => 0
        }
    "#;

    let (_, result) = run_and_get_value(source);
    assert_eq!(result, Value::Number(60.0));
}

#[test]
fn test_guard_prevents_match() {
    let source = r#"
        let x = 42
        match x {
            42 if false => "never",
            42 => "always",
            _ => "other"
        }
    "#;

    let (bytecode, result) = run_and_get_value(source);
    let mut pool = bytecode.string_pool.borrow_mut();
    let always_id = pool.intern("always");
    assert_eq!(result, Value::String(always_id));
}
