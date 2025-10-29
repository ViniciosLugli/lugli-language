mod helpers;
use helpers::run_test;

#[test]
fn test_expression_statements_in_loop_no_stack_pollution() {
    // This was a bug where expression statements inside loops would leave values on the stack
    let source = r#"
        let x = 0

        loop {
            x + 1  # Expression statement that should be popped
            x = x + 1
            if x >= 3 {
                break
            }
        }

        # Stack should be clean here
        let result = 42

        if result != 42 {
            let error = 1 / 0  # Stack was polluted by loop expression statements
        }
    "#;

    run_test(source);
}

#[test]
fn test_method_calls_in_loop_return_correct_values() {
    // This was a bug where method calls in loops were returning null
    let source = r#"
        let words = ["hello", "world"]
        let results = []

        for word in words {
            let upper = word.upper()  # This was returning null
            results.push(upper)
        }

        if results[0] != "HELLO" {
            let error = 1 / 0  # First method call in loop failed
        }

        if results[1] != "WORLD" {
            let error = 1 / 0  # Second method call in loop failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_locals_vs_globals_in_loops() {
    let source = r#"
        let global_var = 0

        fn test_loop() {
            let local_var = 0

            while local_var < 3 {
                global_var = global_var + 1
                local_var = local_var + 1
            }

            return local_var
        }

        let result = test_loop()

        if result != 3 {
            let error = 1 / 0  # Local variable in loop failed
        }

        if global_var != 3 {
            let error = 1 / 0  # Global variable update in loop failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_multiple_method_calls_in_single_statement() {
    let source = r#"
        let text = "  hello  "
        let i = 0

        while i < 3 {
            # Multiple method calls in one statement
            let result = text.trim().upper()

            if result != "HELLO" {
                let error = 1 / 0  # Chained method calls in loop failed
            }

            i = i + 1
        }
    "#;

    run_test(source);
}

#[test]
fn test_stack_restored_after_break() {
    let source = r#"
        let count = 0

        fn loop_with_break() {
            let local = 10

            loop {
                count = count + 1
                if count > 2 {
                    break
                }
                local + count  # Expression that should not pollute
            }

            return local  # Should still have access to local
        }

        let result = loop_with_break()

        if result != 10 {
            let error = 1 / 0  # Stack not properly restored after break
        }

        if count != 3 {
            let error = 1 / 0  # Loop counter incorrect
        }
    "#;

    run_test(source);
}
