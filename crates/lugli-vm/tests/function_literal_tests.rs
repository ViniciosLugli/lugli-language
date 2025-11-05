mod helpers;
use helpers::run_test;

#[test]
fn test_simple_function_literal_stored_in_variable() {
    let source = r#"
        let double = fn(x) { return x * 2 }
        let result = double(21)
        if result != 42 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_function_literal_no_params() {
    let source = r#"
        let get_value = fn() { return 42 }
        let result = get_value()
        if result != 42 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_function_literal_multiple_params() {
    let source = r#"
        let add = fn(a, b, c) { return a + b + c }
        let result = add(10, 20, 30)
        if result != 60 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_function_literal_immediate_call() {
    let source = r#"
        let result = fn(x) { return x + 10 }(32)
        if result != 42 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_function_literal_with_local_variables() {
    let source = r#"
        let compute = fn(x) {
            let doubled = x * 2
            let incremented = doubled + 2
            return incremented
        }
        let result = compute(20)
        if result != 42 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_function_literal_as_callback() {
    let source = r#"
        let numbers = [1, 2, 3, 4, 5]
        let doubled = numbers.map(fn(x) { return x * 2 })

        if doubled[0] != 2 {
            let error = 1 / 0
        }
        if doubled[4] != 10 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_function_literal_filter() {
    let source = r#"
        let numbers = [1, 2, 3, 4, 5]
        let evens = numbers.filter(fn(x) { return x % 2 == 0 })

        if evens.len() != 2 {
            let error = 1 / 0
        }
        if evens[0] != 2 {
            let error = 1 / 0
        }
        if evens[1] != 4 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_nested_function_literals() {
    let source = r#"
        let outer = fn(x) {
            let inner = fn(y) { return y + 10 }
            return inner(x)
        }

        let result = outer(32)
        if result != 42 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_function_literal_returns_function() {
    let source = r#"
        let make_adder = fn(n) {
            return fn(x) { return x + n }
        }

        let add10 = make_adder(10)
        let result = add10(32)
        if result != 42 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}
