mod helpers;

use helpers::{assert_execution_succeeds, run_test};
use lugli_common::Value;

#[test]
fn test_pre_increment_value() {
    let source = r#"
        mut x = 5
        ++x
    "#;
    assert_execution_succeeds(source, Value::Number(6.0));
}

#[test]
fn test_pre_decrement_value() {
    let source = r#"
        mut x = 5
        --x
    "#;
    assert_execution_succeeds(source, Value::Number(4.0));
}

#[test]
fn test_post_increment_value() {
    let source = r#"
        mut x = 5
        x++
    "#;
    assert_execution_succeeds(source, Value::Number(5.0));
}

#[test]
fn test_post_decrement_value() {
    let source = r#"
        mut x = 5
        x--
    "#;
    assert_execution_succeeds(source, Value::Number(5.0));
}

#[test]
fn test_pre_increment_modifies_variable() {
    run_test(
        r#"
        mut x = 5
        let result = ++x
        if x == 6 && result == 6 {
            print("Pre-increment works correctly")
        }
    "#,
    );
}

#[test]
fn test_pre_decrement_modifies_variable() {
    run_test(
        r#"
        mut x = 5
        let result = --x
        if x == 4 && result == 4 {
            print("Pre-decrement works correctly")
        }
    "#,
    );
}

#[test]
fn test_post_increment_modifies_variable() {
    run_test(
        r#"
        mut x = 5
        let result = x++
        if x == 6 && result == 5 {
            print("Post-increment works correctly")
        }
    "#,
    );
}

#[test]
fn test_post_decrement_modifies_variable() {
    run_test(
        r#"
        mut x = 5
        let result = x--
        if x == 4 && result == 5 {
            print("Post-decrement works correctly")
        }
    "#,
    );
}

#[test]
fn test_multiple_increments() {
    run_test(
        r#"
        mut x = 0
        x++
        x++
        x++
        if x == 3 {
            print("Multiple increments work")
        }
    "#,
    );
}

#[test]
fn test_increment_in_expression() {
    run_test(
        r#"
        mut x = 5
        let result = x++ + 10
        if result == 15 && x == 6 {
            print("Post-increment in expression works")
        }
    "#,
    );
}

#[test]
fn test_pre_increment_in_expression() {
    run_test(
        r#"
        mut x = 5
        let result = ++x + 10
        if result == 16 && x == 6 {
            print("Pre-increment in expression works")
        }
    "#,
    );
}

#[test]
fn test_decrement_in_expression() {
    run_test(
        r#"
        mut x = 5
        let result = x-- + 10
        if result == 15 && x == 4 {
            print("Post-decrement in expression works")
        }
    "#,
    );
}

#[test]
fn test_increment_in_loop() {
    run_test(
        r#"
        mut i = 0
        mut count = 0
        while i < 5 {
            count = count + 1
            i++
        }
        if i == 5 && count == 5 {
            print("Increment in loop works")
        }
    "#,
    );
}

#[test]
fn test_decrement_in_loop() {
    run_test(
        r#"
        mut i = 5
        mut count = 0
        while i > 0 {
            count = count + 1
            i--
        }
        if i == 0 && count == 5 {
            print("Decrement in loop works")
        }
    "#,
    );
}

#[test]
fn test_increment_with_negative_numbers() {
    run_test(
        r#"
        mut x = -5
        x++
        if x == -4 {
            print("Increment from negative works")
        }

        mut y = -1
        y++
        if y == 0 {
            print("Increment from -1 to 0 works")
        }
    "#,
    );
}

#[test]
fn test_decrement_with_negative_numbers() {
    run_test(
        r#"
        mut x = -5
        x--
        if x == -6 {
            print("Decrement to more negative works")
        }

        mut y = 1
        y--
        if y == 0 {
            print("Decrement from 1 to 0 works")
        }
    "#,
    );
}

#[test]
fn test_mixed_increment_decrement() {
    run_test(
        r#"
        mut x = 10
        x++
        x--
        x++
        ++x
        --x

        if x == 11 {
            print("Mixed increment/decrement works")
        }
    "#,
    );
}

#[test]
fn test_chained_operations() {
    run_test(
        r#"
        mut x = 0
        mut y = 0

        let a = x++
        let b = ++y

        if x == 1 && y == 1 && a == 0 && b == 1 {
            print("Chained operations work correctly")
        }
    "#,
    );
}

#[test]
fn test_increment_in_function() {
    run_test(
        r#"
        fn counter() {
            mut count = 0
            count++
            return count
        }

        let result1 = counter()
        let result2 = counter()

        if result1 == 1 && result2 == 1 {
            print("Increment in function works (new scope each call)")
        }
    "#,
    );
}

#[test]
fn test_increment_preserves_type() {
    run_test(
        r#"
        mut x = 5
        x++
        let result = x + 0.5
        if result == 6.5 {
            print("Increment preserves numeric type")
        }
    "#,
    );
}

#[test]
fn test_decrement_to_zero() {
    run_test(
        r#"
        mut x = 1
        x--
        if x == 0 {
            print("Decrement to zero works")
        }
    "#,
    );
}

#[test]
fn test_pre_increment_with_comparison() {
    run_test(
        r#"
        mut x = 5
        if ++x > 5 {
            print("Pre-increment in comparison works")
        }
        if x == 6 {
            print("Variable was modified")
        }
    "#,
    );
}

#[test]
fn test_post_increment_with_comparison() {
    run_test(
        r#"
        mut x = 5
        if x++ == 5 {
            print("Post-increment in comparison returns old value")
        }
        if x == 6 {
            print("Variable was modified")
        }
    "#,
    );
}

#[test]
fn test_increment_local_variable() {
    run_test(
        r#"
        fn test() {
            let x = 5
            let result = ++x
            return result
        }
        let val = test()
        if val == 6 {
            print("Local variable increment works")
        }
    "#,
    );
}

#[test]
fn test_increment_statement_only() {
    run_test(
        r#"
        mut x = 10
        ++x
        if x == 11 {
            print("Statement-only pre-increment works")
        }
    "#,
    );
}

#[test]
fn test_post_increment_statement_only() {
    run_test(
        r#"
        mut x = 10
        x++
        if x == 11 {
            print("Statement-only post-increment works")
        }
    "#,
    );
}
