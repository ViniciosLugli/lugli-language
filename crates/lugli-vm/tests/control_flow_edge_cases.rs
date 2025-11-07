mod helpers;
use helpers::run_test;

// Empty block tests
#[test]
fn test_empty_if_block() {
    run_test(
        r#"
        let x = 5
        if x > 0 {
        }
        if x == 5 {
            let y = 10
        }
    "#,
    );
}

#[test]
fn test_empty_else_block() {
    run_test(
        r#"
        let x = 5
        if x < 0 {
            let y = 10
        } else {
        }
    "#,
    );
}

#[test]
fn test_empty_loop_with_break() {
    run_test(
        r#"
        loop {
            break
        }
    "#,
    );
}

// Match edge cases
#[test]
fn test_match_all_guards_fail() {
    run_test(
        r#"
        let x = 5
        let result = match x {
            n if n < 0 => "negative",
            n if n > 10 => "large",
            _ => "default"
        }
        if result != "default" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_match_single_arm() {
    run_test(
        r#"
        let x = 5
        let result = match x {
            _ => "any"
        }
        if result != "any" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_match_with_null() {
    run_test(
        r#"
        let x = null
        let result = match x {
            null => "is null",
            _ => "not null"
        }
        if result != "is null" {
            let error = 1 / 0
        }
    "#,
    );
}

// For loop edge cases
#[test]
fn test_for_loop_empty_list() {
    run_test(
        r#"
        let count = 0
        for item in [] {
            count = count + 1
        }
        if count != 0 {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_for_loop_single_item() {
    run_test(
        r#"
        let count = 0
        for item in [42] {
            count = count + 1
        }
        if count != 1 {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_for_loop_break_immediately() {
    run_test(
        r#"
        let count = 0
        for item in [1, 2, 3, 4, 5] {
            break
            count = count + 1
        }
        if count != 0 {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_for_loop_continue_all() {
    run_test(
        r#"
        let count = 0
        for item in [1, 2, 3] {
            continue
            count = count + 1
        }
        if count != 0 {
            let error = 1 / 0
        }
    "#,
    );
}

// While loop edge cases
#[test]
fn test_while_loop_never_executes() {
    run_test(
        r#"
        let x = 10
        let count = 0
        while x < 5 {
            count = count + 1
        }
        if count != 0 {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_while_loop_immediate_break() {
    run_test(
        r#"
        let count = 0
        while true {
            break
            count = count + 1
        }
        if count != 0 {
            let error = 1 / 0
        }
    "#,
    );
}

// Nested control flow
#[test]
fn test_nested_if_else() {
    run_test(
        r#"
        let x = 5
        let y = 10
        let result = if x > 0 {
            if y > 5 {
                "both positive"
            } else {
                "x positive"
            }
        } else {
            "x not positive"
        }
        if result != "both positive" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_nested_loops_with_break() {
    run_test(
        r#"
        let outer_count = 0
        for i in [1, 2, 3] {
            outer_count = outer_count + 1
            for j in [1, 2] {
                if j == 2 {
                    break
                }
            }
        }
        if outer_count != 3 {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_nested_loops_with_continue() {
    run_test(
        r#"
        let count = 0
        for i in [1, 2] {
            for j in [1, 2, 3] {
                if j == 2 {
                    continue
                }
                count = count + 1
            }
        }
        if count != 4 {
            let error = 1 / 0
        }
    "#,
    );
}

// Boolean short-circuit
#[test]
fn test_and_short_circuit() {
    run_test(
        r#"
        let x = false
        let called = false

        fn set_called() {
            called = true
            return true
        }

        # Should not call set_called
        if x && set_called() {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_or_short_circuit() {
    run_test(
        r#"
        let x = true
        let called = false

        fn set_called() {
            called = true
            return false
        }

        # Should not call set_called
        if x || set_called() {
            # Success
        }
    "#,
    );
}

// Null handling
#[test]
fn test_null_in_if_condition() {
    run_test(
        r#"
        let x = null
        if x {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_null_equality() {
    run_test(
        r#"
        let x = null
        let y = null
        if x != y {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_null_in_list() {
    run_test(
        r#"
        let list = [1, null, 3]
        if list[1] != null {
            let error = 1 / 0
        }
    "#,
    );
}

// Boundary conditions
#[test]
fn test_division_by_very_small_number() {
    run_test(
        r#"
        let x = 10 / 0.0001
        if x != 100000 {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_large_number_arithmetic() {
    run_test(
        r#"
        let x = 1000000 * 1000000
        if x != 1000000000000 {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_deep_nesting() {
    run_test(
        r#"
        fn deeply_nested() {
            if true {
                if true {
                    if true {
                        if true {
                            if true {
                                return "deep"
                            }
                        }
                    }
                }
            }
            return "wrong"
        }

        let result = deeply_nested()
        if result != "deep" {
            let error = 1 / 0
        }
    "#,
    );
}

// Edge cases with empty strings and collections
#[test]
fn test_empty_string_comparison() {
    run_test(
        r#"
        let s = ""
        if s == "" {
            # Empty string comparison works
        } else {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_empty_list_length() {
    run_test(
        r#"
        let list = []
        if list.len() != 0 {
            let error = 1 / 0
        }
        if !list.is_empty() {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_empty_dict_length() {
    run_test(
        r#"
        let dict = {}
        if dict.len() != 0 {
            let error = 1 / 0
        }
    "#,
    );
}

// Type comparison edge cases
#[test]
fn test_compare_different_types() {
    run_test(
        r#"
        let x = 5
        let y = "5"
        if x == y {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_compare_bool_with_number() {
    run_test(
        r#"
        let x = true
        let y = 1
        if x == y {
            let error = 1 / 0
        }
    "#,
    );
}

// Return in different contexts
#[test]
fn test_return_in_nested_if() {
    run_test(
        r#"
        fn test_func() {
            if true {
                if true {
                    return 42
                }
            }
            return 0
        }

        if test_func() != 42 {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_return_in_loop() {
    run_test(
        r#"
        fn find_item() {
            for i in [1, 2, 3, 4, 5] {
                if i == 3 {
                    return i
                }
            }
            return -1
        }

        if find_item() != 3 {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_early_return() {
    run_test(
        r#"
        fn test_func(x) {
            if x < 0 {
                return "negative"
            }
            return "non-negative"
        }

        if test_func(-5) != "negative" {
            let error = 1 / 0
        }
        if test_func(5) != "non-negative" {
            let error = 1 / 0
        }
    "#,
    );
}
