// Short-circuit evaluation tests - validates && and || operators properly short-circuit
//
// Tests that:
// - a && b does NOT evaluate b if a is false
// - a || b does NOT evaluate b if a is true
// - Side effects are properly preserved
// - Return values are correct
// - Nested and chained short-circuits work correctly

mod helpers;

use helpers::{assert_execution_succeeds, run_test};
use lugli_common::Value;

#[test]
fn test_and_short_circuit_basic() {
    // false && X should NOT evaluate X
    assert_execution_succeeds("false && true", Value::Bool(false));
    assert_execution_succeeds("false && false", Value::Bool(false));

    // true && X SHOULD evaluate X
    assert_execution_succeeds("true && true", Value::Bool(true));
    assert_execution_succeeds("true && false", Value::Bool(false));
}

#[test]
fn test_or_short_circuit_basic() {
    // true || X should NOT evaluate X
    assert_execution_succeeds("true || true", Value::Bool(true));
    assert_execution_succeeds("true || false", Value::Bool(true));

    // false || X SHOULD evaluate X
    assert_execution_succeeds("false || true", Value::Bool(true));
    assert_execution_succeeds("false || false", Value::Bool(false));
}

#[test]
fn test_and_short_circuit_side_effects() {
    // Side effect should NOT execute when short-circuited
    let source = r#"
        mut counter = 0

        fn increment() {
            counter = counter + 1
            return true
        }

        let result = false && increment()

        if counter != 0 {
            return "FAIL: increment() was called when it shouldn't be"
        }
        if result != false {
            return "FAIL: result should be false"
        }
    "#;
    run_test(source);
}

#[test]
fn test_and_no_short_circuit_side_effects() {
    // Side effect SHOULD execute when not short-circuited
    let source = r#"
        mut counter = 0

        fn increment() {
            counter = counter + 1
            return true
        }

        let result = true && increment()

        if counter != 1 {
            return "FAIL: increment() was not called when it should be"
        }
        if result != true {
            return "FAIL: result should be true"
        }
    "#;
    run_test(source);
}

#[test]
fn test_or_short_circuit_side_effects() {
    // Side effect should NOT execute when short-circuited
    let source = r#"
        mut counter = 0

        fn increment() {
            counter = counter + 1
            return false
        }

        let result = true || increment()

        if counter != 0 {
            return "FAIL: increment() was called when it shouldn't be"
        }
        if result != true {
            return "FAIL: result should be true"
        }
    "#;
    run_test(source);
}

#[test]
fn test_or_no_short_circuit_side_effects() {
    // Side effect SHOULD execute when not short-circuited
    let source = r#"
        mut counter = 0

        fn increment() {
            counter = counter + 1
            return false
        }

        let result = false || increment()

        if counter != 1 {
            return "FAIL: increment() was not called when it should be"
        }
        if result != false {
            return "FAIL: result should be false"
        }
    "#;
    run_test(source);
}

#[test]
fn test_and_return_values() {
    // && returns first falsy value or last value
    assert_execution_succeeds("0 && 5", Value::Number(0.0));
    assert_execution_succeeds("false && true", Value::Bool(false));
    assert_execution_succeeds("null && 5", Value::Null);
    assert_execution_succeeds("1 && 2", Value::Number(2.0));
    assert_execution_succeeds("5 && 10", Value::Number(10.0));
}

#[test]
fn test_or_return_values() {
    // || returns first truthy value or last value
    assert_execution_succeeds("0 || 5", Value::Number(5.0));
    assert_execution_succeeds("false || true", Value::Bool(true));
    assert_execution_succeeds("null || 42", Value::Number(42.0));
    assert_execution_succeeds("1 || 2", Value::Number(1.0));
    assert_execution_succeeds("5 || 10", Value::Number(5.0));
    assert_execution_succeeds("0 || false", Value::Bool(false));
}

#[test]
fn test_chained_and_operations() {
    // Multiple && in sequence
    assert_execution_succeeds("true && true && true", Value::Bool(true));
    assert_execution_succeeds("true && false && true", Value::Bool(false));
    assert_execution_succeeds("false && true && true", Value::Bool(false));

    let source = r#"
        mut counter = 0

        fn increment() {
            counter = counter + 1
            return true
        }

        # First false should stop all subsequent calls
        let result = false && increment() && increment() && increment()

        if counter != 0 {
            return "FAIL: functions were called when they shouldn't be"
        }
    "#;
    run_test(source);
}

#[test]
fn test_chained_or_operations() {
    // Multiple || in sequence
    assert_execution_succeeds("false || false || true", Value::Bool(true));
    assert_execution_succeeds("false || true || false", Value::Bool(true));
    assert_execution_succeeds("true || false || false", Value::Bool(true));

    let source = r#"
        mut counter = 0

        fn increment() {
            counter = counter + 1
            return false
        }

        # First true should stop all subsequent calls
        let result = true || increment() || increment() || increment()

        if counter != 0 {
            return "FAIL: functions were called when they shouldn't be"
        }
    "#;
    run_test(source);
}

#[test]
fn test_nested_short_circuits() {
    let source = r#"
        mut a_called = false
        mut b_called = false
        mut c_called = false

        fn set_a() {
            a_called = true
            return false
        }

        fn set_b() {
            b_called = true
            return true
        }

        fn set_c() {
            c_called = true
            return true
        }

        # (false && b()) || c() should call a and c, but not b
        let result = (set_a() && set_b()) || set_c()

        if !a_called {
            return "FAIL: a should be called"
        }
        if b_called {
            return "FAIL: b should not be called (short-circuited by false)"
        }
        if !c_called {
            return "FAIL: c should be called"
        }
        if result != true {
            return "FAIL: result should be true"
        }
    "#;
    run_test(source);
}

#[test]
fn test_short_circuit_with_comparisons() {
    let source = r#"
        mut counter = 0

        fn increment() {
            counter = counter + 1
            return 42
        }

        # 5 > 10 is false, so increment() should not be called
        let result = 5 > 10 && increment() > 0

        if counter != 0 {
            return "FAIL: increment() was called when it shouldn't be"
        }
        if result != false {
            return "FAIL: result should be false"
        }
    "#;
    run_test(source);
}

#[test]
fn test_short_circuit_in_conditionals() {
    let source = r#"
        mut counter = 0

        fn increment() {
            counter = counter + 1
            return true
        }

        if false && increment() {
            return "FAIL: should not enter if block"
        }

        if counter != 0 {
            return "FAIL: increment() was called in if condition"
        }

        if true || increment() {
            # Should enter, but increment() should not be called
            if counter != 0 {
                return "FAIL: increment() was called in second if"
            }
        } else {
            return "FAIL: should enter if block"
        }
    "#;
    run_test(source);
}

#[test]
fn test_short_circuit_mixed_operators() {
    // Test mixing && and || with proper precedence
    assert_execution_succeeds("true && false || true", Value::Bool(true));
    assert_execution_succeeds("false || true && false", Value::Bool(false));
    assert_execution_succeeds("true && true || false", Value::Bool(true));

    let source = r#"
        mut counter = 0

        fn increment() {
            counter = counter + 1
            return true
        }

        # true && false short-circuits to false, then false || increment() evaluates increment
        let result = true && false || increment()

        if counter != 1 {
            return "FAIL: increment() should be called once"
        }
        if result != true {
            return "FAIL: result should be true"
        }
    "#;
    run_test(source);
}

#[test]
fn test_short_circuit_with_function_calls() {
    let source = r#"
        fn always_true() {
            return true
        }

        fn always_false() {
            return false
        }

        mut expensive_called = false

        fn expensive_operation() {
            expensive_called = true
            return 42
        }

        # Should short-circuit before expensive_operation
        let result = always_false() && expensive_operation() > 0

        if expensive_called {
            return "FAIL: expensive_operation was called"
        }

        # Should short-circuit before expensive_operation
        let result2 = always_true() || expensive_operation() > 0

        if expensive_called {
            return "FAIL: expensive_operation was called on second test"
        }
    "#;
    run_test(source);
}

#[test]
fn test_short_circuit_preserves_evaluation_order() {
    let source = r#"
        mut order = []

        fn mark_a() {
            order.push("a")
            return true
        }

        fn mark_b() {
            order.push("b")
            return false
        }

        fn mark_c() {
            order.push("c")
            return true
        }

        # Test: a() && b() && c() should call a, b, but not c
        let result = mark_a() && mark_b() && mark_c()

        if order.len() != 2 {
            return "FAIL: wrong number of calls"
        }
        if order[0] != "a" {
            return "FAIL: first should be 'a'"
        }
        if order[1] != "b" {
            return "FAIL: second should be 'b'"
        }
    "#;
    run_test(source);
}

#[test]
fn test_short_circuit_complex_expressions() {
    let source = r#"
        mut operations = 0

        fn count() {
            operations = operations + 1
            return operations
        }

        # Complex: (a && b) || (c && d)
        # If a && b succeeds, c && d should not execute
        let result = (count() > 0 && count() > 0) || (count() > 0 && count() > 0)

        if operations != 2 {
            return "FAIL: should only call count() twice (a && b succeeds)"
        }
        if result != true {
            return "FAIL: result should be true"
        }
    "#;
    run_test(source);
}

#[test]
fn test_short_circuit_with_nullish_values() {
    // Test with null, 0 (both falsy in Lugli)
    assert_execution_succeeds("null && 5", Value::Null);
    assert_execution_succeeds("0 && 5", Value::Number(0.0));

    let source = r#"
        mut called = false

        fn side_effect() {
            called = true
            return 10
        }

        let result = null || side_effect()

        if !called {
            return "FAIL: side_effect should be called"
        }
        if result != 10 {
            return "FAIL: result should be 10"
        }
    "#;
    run_test(source);
}
