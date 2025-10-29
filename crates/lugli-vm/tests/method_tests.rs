// Method tests - validates built-in methods on values
//
// Tests string, list, and dict methods including:
// - String methods: upper, lower, split, join, trim
// - List methods: push!, pop!, filter, map!, len
// - Dict methods: get, keys, values
// - Method chaining
// - Methods in control flow
// - Error handling for invalid method calls

mod helpers;
use helpers::{run_test, run_test_expect_error};

#[test]
fn test_string_methods() {
    // Non-mutating methods
    let source = r#"
        let text = "hello world"
        let upper = text.upper()
        let lower = text.lower()

        if upper != "HELLO WORLD" {
            let error = 1 / 0  # upper() failed
        }

        if lower != "hello world" {
            let error = 1 / 0  # lower() failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_list_mutating_methods() {
    let source = r#"
        let list = [1, 2, 3]
        list.push!(4)

        if list.len() != 4 {
            let error = 1 / 0  # push! failed
        }

        if list[3] != 4 {
            let error = 1 / 0  # push! didn't add element correctly
        }

        let popped = list.pop!()
        if popped != 4 {
            let error = 1 / 0  # pop! returned wrong value
        }

        if list.len() != 3 {
            let error = 1 / 0  # pop! didn't remove element
        }
    "#;

    run_test(source);
}

#[test]
fn test_list_non_mutating_methods() {
    let source = r#"
        let list = [1, 2, 3, 4, 5]

        # Filter even numbers
        let evens = list.filter(fn(x) { x % 2 == 0 })
        if evens.len() != 2 {
            let error = 1 / 0  # filter failed
        }
        if evens[0] != 2 || evens[1] != 4 {
            let error = 1 / 0  # filter returned wrong values
        }

        # Map to double values
        let doubled = list.map!(fn(x) { x * 2 })
        if doubled[0] != 2 || doubled[4] != 10 {
            let error = 1 / 0  # map! failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_dict_methods() {
    let source = r#"
        let dict = {"a": 1, "b": 2, "c": 3}

        # Get value
        if dict.get("b") != 2 {
            let error = 1 / 0  # get() failed
        }

        # Get with default
        if dict.get("d", 0) != 0 {
            let error = 1 / 0  # get() with default failed
        }

        # Keys and values
        let keys = dict.keys()
        let values = dict.values()

        if keys.len() != 3 {
            let error = 1 / 0  # keys() failed
        }

        if values.len() != 3 {
            let error = 1 / 0  # values() failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_method_chaining() {
    let source = r#"
        let text = "  hello world  "
        let result = text.trim().upper().split(" ")

        if result.len() != 2 {
            let error = 1 / 0  # Method chaining failed - wrong length
        }

        if result[0] != "HELLO" {
            let error = 1 / 0  # Method chaining failed - first element wrong
        }

        if result[1] != "WORLD" {
            let error = 1 / 0  # Method chaining failed - second element wrong
        }
    "#;

    run_test(source);
}

#[test]
fn test_methods_in_control_flow() {
    let source = r#"
        let list = []
        let i = 0

        while i < 5 {
            list.push!(i)
            i = i + 1
        }

        if list.len() != 5 {
            let error = 1 / 0  # Methods in while loop failed
        }

        for item in list {
            if item >= 0 && item < 5 {
                # OK
            } else {
                let error = 1 / 0  # Methods produced wrong values in loop
            }
        }
    "#;

    run_test(source);
}

#[test]
fn test_method_with_multiple_arguments() {
    let source = r#"
        let text = "a,b,c,d"
        let parts = text.split(",")

        if parts.len() != 4 {
            let error = 1 / 0  # split() failed
        }

        let joined = parts.join("-")
        if joined != "a-b-c-d" {
            let error = 1 / 0  # join() failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_methods_on_literals() {
    let source = r#"
        # Methods directly on literals
        let upper = "hello".upper()
        if upper != "HELLO" {
            let error = 1 / 0  # Method on string literal failed
        }

        let len = [1, 2, 3].len()
        if len != 3 {
            let error = 1 / 0  # Method on list literal failed
        }

        let value = {"key": "value"}.get("key")
        if value != "value" {
            let error = 1 / 0  # Method on dict literal failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_nested_method_calls() {
    let source = r#"
        let data = {
            "users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ]
        }

        let users = data.get("users")
        let first = users[0]
        let name = first.get("name")

        if name != "Alice" {
            let error = 1 / 0  # Nested method access failed
        }

        # More complex nesting
        if data.get("users")[1].get("age") != 25 {
            let error = 1 / 0  # Complex nested method access failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_methods_with_closures() {
    let source = r#"
        let numbers = [1, 2, 3, 4, 5]

        # Filter with closure capturing external variable
        let threshold = 3
        let filtered = numbers.filter(fn(x) { x > threshold })

        if filtered.len() != 2 {
            let error = 1 / 0  # Filter with closure failed
        }

        # Map with closure
        let factor = 10
        let scaled = numbers.map!(fn(x) { x * factor })

        if scaled[0] != 10 || scaled[4] != 50 {
            let error = 1 / 0  # Map with closure failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_method_error_handling() {
    // Test that methods fail gracefully with wrong types
    let source = r#"
        let num = 42
        # This should fail at runtime - numbers don't have upper() method
        let result = num.upper()
    "#;

    assert!(run_test_expect_error(source), "Should fail when calling string method on number");

    // Test with wrong number of arguments
    let source2 = r#"
        let text = "hello"
        # split() requires an argument
        let parts = text.split()
    "#;

    assert!(run_test_expect_error(source2), "Should fail with wrong number of arguments");
}

#[test]
fn test_mutating_vs_non_mutating() {
    let source = r#"
        let original = [1, 2, 3]

        # Non-mutating method - original unchanged
        let filtered = original.filter(fn(x) { x > 1 })
        if original.len() != 3 {
            let error = 1 / 0  # Filter mutated original list
        }

        # Mutating method - original changed
        original.push!(4)
        if original.len() != 4 {
            let error = 1 / 0  # Push! didn't mutate original list
        }

        # Verify original still has all elements
        if original[0] != 1 || original[3] != 4 {
            let error = 1 / 0  # List corruption after mutations
        }
    "#;

    run_test(source);
}

#[test]
fn test_method_return_values() {
    let source = r#"
        fn get_data() {
            return [1, 2, 3]
        }

        fn get_text() {
            return "hello"
        }

        # Methods on function return values
        if get_data().len() != 3 {
            let error = 1 / 0  # Method on function return failed
        }

        if get_text().upper() != "HELLO" {
            let error = 1 / 0  # Method on string return failed
        }

        # Chain methods on returns
        if get_data().filter(fn(x) { x > 1 }).len() != 2 {
            let error = 1 / 0  # Chained methods on return failed
        }
    "#;

    run_test(source);
}
