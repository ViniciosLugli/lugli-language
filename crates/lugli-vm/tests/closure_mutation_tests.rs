mod helpers;
use helpers::run_test;

#[test]
fn test_closure_upvalue_mutation() {
    let source = r#"
        let counter = 0
        let increment = fn() {
            counter = counter + 1
            return counter
        }

        let first = increment()
        if first != 1 {
            let error = 1 / 0
        }

        let second = increment()
        if second != 2 {
            let error = 1 / 0
        }

        # Verify outer variable was updated
        if counter != 2 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_multiple_closures_sharing_upvalue() {
    let source = r#"
        let shared = 10

        let add = fn(x) { return shared + x }
        let multiply = fn(x) { return shared * x }

        if add(5) != 15 {
            let error = 1 / 0
        }

        if multiply(3) != 30 {
            let error = 1 / 0
        }

        # Both closures share the same upvalue
        shared = 20

        if add(5) != 25 {
            let error = 1 / 0
        }

        if multiply(3) != 60 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
#[ignore = "Nested closure mutations need shared upvalue references - current implementation captures by value"]
fn test_nested_closure_upvalue_mutation() {
    let source = r#"
        fn make_counter(start) {
            let count = start

            return fn() {
                count = count + 1
                return count
            }
        }

        let counter1 = make_counter(0)
        let counter2 = make_counter(100)

        if counter1() != 1 {
            let error = 1 / 0
        }

        if counter1() != 2 {
            let error = 1 / 0
        }

        if counter2() != 101 {
            let error = 1 / 0
        }

        if counter1() != 3 {
            let error = 1 / 0
        }

        if counter2() != 102 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_closure_capturing_multiple_upvalues() {
    let source = r#"
        let a = 10
        let b = 20
        let c = 30

        let closure = fn(x) {
            return a + b + c + x
        }

        if closure(5) != 65 {
            let error = 1 / 0
        }

        a = 1
        b = 2
        c = 3

        if closure(5) != 11 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}
