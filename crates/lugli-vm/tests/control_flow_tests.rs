mod helpers;
use helpers::run_test;

#[test]
fn test_simple_while_loop() {
    let source = r#"
        let count = 0
        let sum = 0

        while count < 5 {
            sum = sum + count
            count = count + 1
        }

        # Assert sum == 10 (0 + 1 + 2 + 3 + 4 = 10)
        if sum != 10 {
            let error = 1 / 0  # Trigger error if assertion fails
        }
    "#;

    run_test(source);
}

#[test]
fn test_if_elif_else_chain() {
    let source = r#"
        let score = 75
        let grade = ""

        if score >= 90 {
            grade = "A"
        } elif score >= 80 {
            grade = "B"
        } elif score >= 70 {
            grade = "C"
        } else {
            grade = "F"
        }

        if grade != "C" {
            let error = 1 / 0  # If-elif-else chain failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_infinite_loop_with_break() {
    let source = r#"
        let counter = 0

        loop {
            counter = counter + 1
            if counter >= 3 {
                break
            }
        }

        if counter != 3 {
            let error = 1 / 0  # Loop with break failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_loop_with_continue() {
    let source = r#"
        let sum = 0
        let i = 0

        loop {
            i = i + 1
            if i > 6 {
                break
            }
            if i % 2 == 1 {
                continue  # Skip odd numbers
            }
            sum = sum + i
        }

        if sum != 12 {  # 2 + 4 + 6 = 12
            let error = 1 / 0  # Loop with continue failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_nested_loops_with_break() {
    let source = r#"
        let result = 0

        for i in range(3) {
            for j in range(3) {
                result = result + 1
            }
        }

        if result != 9 {  # 3 * 3 = 9
            let error = 1 / 0  # Nested loops count incorrect
        }

        # Test with break in inner loop
        let count = 0
        for i in range(3) {
            for j in range(5) {
                if j == 2 {
                    break  # Break inner loop only
                }
                count = count + 1
            }
        }

        if count != 6 {  # 3 iterations * 2 inner iterations each
            let error = 1 / 0  # Nested loops with break failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_complex_control_flow() {
    let source = r#"
        let result = []

        for i in range(5) {
            if i == 0 {
                result.push("zero")
            } elif i % 2 == 0 {
                result.push("even")
            } else {
                result.push("odd")
            }
        }

        if result.len() != 5 {
            let error = 1 / 0  # Complex control flow failed
        }

        if result[0] != "zero" {
            let error = 1 / 0  # First element wrong
        }

        if result[1] != "odd" {
            let error = 1 / 0  # Second element wrong
        }

        if result[2] != "even" {
            let error = 1 / 0  # Third element wrong
        }
    "#;

    run_test(source);
}
