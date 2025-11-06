mod helpers;
use helpers::run_test;
use lugli_parser::Parser;
use lugli_vm::compile_and_run;

#[test]
fn test_basic_array_indexing() {
    let source = r#"
        let arr = [10, 20, 30, 40, 50]

        if arr[0] != 10 {
            let error = 1 / 0  # First element access failed
        }

        if arr[2] != 30 {
            let error = 1 / 0  # Middle element access failed
        }

        if arr[4] != 50 {
            let error = 1 / 0  # Last element access failed
        }

        # Negative indexing
        if arr[-1] != 50 {
            let error = 1 / 0  # Negative index -1 failed
        }

        if arr[-2] != 40 {
            let error = 1 / 0  # Negative index -2 failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_dictionary_indexing() {
    let source = r#"
        let dict = {
            "name": "Alice",
            "age": 30,
            "active": true
        }

        if dict["name"] != "Alice" {
            let error = 1 / 0  # String key access failed
        }

        if dict["age"] != 30 {
            let error = 1 / 0  # Number value access failed
        }

        if dict["active"] != true {
            let error = 1 / 0  # Boolean value access failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_string_indexing() {
    let source = r#"
        let text = "Hello"

        if text[0] != "H" {
            let error = 1 / 0  # String index 0 failed
        }

        if text[1] != "e" {
            let error = 1 / 0  # String index 1 failed
        }

        if text[-1] != "o" {
            let error = 1 / 0  # String negative index failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_index_assignment() {
    let source = r#"
        let arr = [1, 2, 3, 4, 5]
        arr[0] = 100
        arr[2] = 300
        arr[-1] = 500

        if arr[0] != 100 {
            let error = 1 / 0  # Index assignment at 0 failed
        }

        if arr[2] != 300 {
            let error = 1 / 0  # Index assignment at 2 failed
        }

        if arr[4] != 500 {
            let error = 1 / 0  # Negative index assignment failed
        }

        # Dictionary assignment
        let dict = {"a": 1, "b": 2}
        dict["a"] = 10
        dict["c"] = 30  # Add new key

        if dict["a"] != 10 {
            let error = 1 / 0  # Dict index assignment failed
        }

        if dict["c"] != 30 {
            let error = 1 / 0  # Dict new key assignment failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_nested_indexing() {
    let source = r#"
        let matrix = [
            [1, 2, 3],
            [4, 5, 6],
            [7, 8, 9]
        ]

        if matrix[0][0] != 1 {
            let error = 1 / 0  # Nested index [0][0] failed
        }

        if matrix[1][1] != 5 {
            let error = 1 / 0  # Nested index [1][1] failed
        }

        if matrix[2][2] != 9 {
            let error = 1 / 0  # Nested index [2][2] failed
        }

        # Nested dictionary
        let data = {
            "users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ]
        }

        if data["users"][0]["name"] != "Alice" {
            let error = 1 / 0  # Complex nested indexing failed
        }

        if data["users"][1]["age"] != 25 {
            let error = 1 / 0  # Complex nested indexing failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_dynamic_index_expressions() {
    let source = r#"
        let arr = [10, 20, 30, 40, 50]
        let i = 2

        if arr[i] != 30 {
            let error = 1 / 0  # Variable index failed
        }

        if arr[i + 1] != 40 {
            let error = 1 / 0  # Expression index failed
        }

        fn get_index() {
            return 3
        }

        if arr[get_index()] != 40 {
            let error = 1 / 0  # Function call index failed
        }

        # Using loop variable as index
        let sum = 0
        for j in [0, 1, 2] {
            sum = sum + arr[j]
        }

        if sum != 60 {
            let error = 1 / 0  # Loop variable index failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_index_in_control_flow() {
    let source = r#"
        let scores = [85, 92, 78, 95, 88]
        let passing = []

        for i in [0, 1, 2, 3, 4] {
            if scores[i] >= 80 {
                passing.push(scores[i])
            }
        }

        if passing.len() != 4 {
            let error = 1 / 0  # Index in if condition failed
        }

        # While loop with index
        let j = 0
        let found = false
        while j < scores.len() {
            if scores[j] == 95 {
                found = true
                break
            }
            j = j + 1
        }

        if !found {
            let error = 1 / 0  # Index in while loop failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_index_with_method_calls() {
    let source = r#"
        let words = ["hello", "world", "test"]

        if words[0].upper() != "HELLO" {
            let error = 1 / 0  # Method call on indexed value failed
        }

        let lengths = []
        for word in words {
            lengths.push(word.len())
        }

        if lengths[1] != 5 {
            let error = 1 / 0  # Index after method result failed
        }

        # Chain indexing and methods
        let data = {
            "items": ["apple", "banana", "cherry"]
        }

        if data["items"][1].upper() != "BANANA" {
            let error = 1 / 0  # Chained index and method failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_index_bounds_checking() {
    // Test out of bounds access returns null
    let source = r#"
        let arr = [1, 2, 3]
        let value = arr[10]

        if value != null {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_slice_operations() {
    let source = r#"
        let arr = [1, 2, 3, 4, 5]

        # Range indexing (if supported)
        # let slice = arr[1:3]
        # This would test slice operations if implemented

        # For now, test manual slicing
        let slice = []
        for i in [1, 2] {
            slice.push(arr[i])
        }

        if slice.len() != 2 {
            let error = 1 / 0  # Manual slice failed
        }

        if slice[0] != 2 || slice[1] != 3 {
            let error = 1 / 0  # Manual slice values incorrect
        }
    "#;

    run_test(source);
}

#[test]
fn test_index_assignment_with_expressions() {
    let source = r#"
        let arr = [0, 0, 0, 0, 0]
        let i = 1

        arr[i] = 10
        arr[i + 1] = 20
        arr[i * 2] = 30

        if arr[1] != 10 {
            let error = 1 / 0  # Variable index assignment failed
        }

        if arr[2] != 30 {
            let error = 1 / 0  # Expression index assignment failed (i+1=2, then i*2=2 overwrites to 30)
        }

        # Dictionary with dynamic keys
        let dict = {}
        let key = "dynamic"
        dict[key] = "value"

        if dict["dynamic"] != "value" {
            let error = 1 / 0  # Dynamic key assignment failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_multidimensional_assignment() {
    let source = r#"
        let matrix = [
            [0, 0, 0],
            [0, 0, 0],
            [0, 0, 0]
        ]

        matrix[0][0] = 1
        matrix[1][1] = 2
        matrix[2][2] = 3

        if matrix[0][0] != 1 {
            let error = 1 / 0  # 2D assignment [0][0] failed
        }

        if matrix[1][1] != 2 {
            let error = 1 / 0  # 2D assignment [1][1] failed
        }

        if matrix[2][2] != 3 {
            let error = 1 / 0  # 2D assignment [2][2] failed
        }

        # Sum diagonal
        let sum = matrix[0][0] + matrix[1][1] + matrix[2][2]
        if sum != 6 {
            let error = 1 / 0  # Diagonal sum failed
        }
    "#;

    run_test(source);
}
#[test]
fn test_negative_index_boundaries() {
    let source = r#"
        let arr = [10, 20, 30]
        
        # Access first element via -len
        if arr[-3] != 10 {
            let error = 1 / 0
        }
        
        # Access middle element
        if arr[-2] != 20 {
            let error = 1 / 0
        }
        
        # Access last element
        if arr[-1] != 30 {
            let error = 1 / 0
        }
        
        # String negative indexing boundaries
        let text = "ABC"
        if text[-3] != "A" {
            let error = 1 / 0
        }
        if text[-1] != "C" {
            let error = 1 / 0
        }
    "#;
    run_test(source);
}

#[test]
fn test_negative_index_out_of_bounds() {
    // Test arr[-4] when len=3 returns null
    let source = r#"
        let arr = [1, 2, 3]
        let val = arr[-4]

        if val != null {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_positive_index_out_of_bounds() {
    // Test arr[3] when len=3 returns null
    let source = r#"
        let arr = [1, 2, 3]
        let val = arr[3]

        if val != null {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_negative_index_assignment_boundaries() {
    let source = r#"
        let arr = [1, 2, 3, 4, 5]
        
        # Assign to first element via -len
        arr[-5] = 100
        if arr[0] != 100 {
            let error = 1 / 0
        }
        
        # Assign to last element
        arr[-1] = 500
        if arr[4] != 500 {
            let error = 1 / 0
        }
        
        # Assign to middle
        arr[-3] = 300
        if arr[2] != 300 {
            let error = 1 / 0
        }
    "#;
    run_test(source);
}

#[test]
fn test_negative_index_assignment_out_of_bounds() {
    let source = r#"
        let arr = [1, 2, 3]
        arr[-4] = 999
    "#;

    let mut parser = Parser::new(source).unwrap();
    let (ast, span_map) = parser.parse().unwrap();
    let result = compile_and_run(&ast, span_map);
    assert!(result.is_err(), "Assignment at index -4 should fail for length 3");
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("out of range") || err_msg.contains("Negative index"), "Error should mention out of range, got: {}", err_msg);
}

#[test]
fn test_float_index_validation_positive() {
    let source = r#"
        let arr = [10, 20, 30]
        let val = arr[1.5]
    "#;

    let mut parser = Parser::new(source).unwrap();
    let (ast, span_map) = parser.parse().unwrap();
    let result = compile_and_run(&ast, span_map);
    assert!(result.is_err(), "Float index 1.5 should be rejected");
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must be an integer"), "Error should mention integer requirement, got: {}", err_msg);
}

#[test]
fn test_float_index_validation_negative() {
    let source = r#"
        let arr = [10, 20, 30]
        let val = arr[-1.5]
    "#;

    let mut parser = Parser::new(source).unwrap();
    let (ast, span_map) = parser.parse().unwrap();
    let result = compile_and_run(&ast, span_map);
    assert!(result.is_err(), "Float index -1.5 should be rejected");
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must be an integer"), "Error should mention integer requirement, got: {}", err_msg);
}

#[test]
fn test_float_index_validation_near_integer() {
    let source = r#"
        let arr = [10, 20, 30]
        let val = arr[1.9999]
    "#;

    let mut parser = Parser::new(source).unwrap();
    let (ast, span_map) = parser.parse().unwrap();
    let result = compile_and_run(&ast, span_map);
    assert!(result.is_err(), "Float index 1.9999 should be rejected");
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must be an integer"), "Error should mention integer requirement, got: {}", err_msg);
}

#[test]
fn test_exact_integer_float_index_works() {
    let source = r#"
        let arr = [10, 20, 30]

        if arr[1.0] != 20 {
            let error = 1 / 0
        }

        if arr[0.0] != 10 {
            let error = 1 / 0
        }

        if arr[-1.0] != 30 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_float_index_assignment_validation() {
    let source = r#"
        let arr = [10, 20, 30]
        arr[1.5] = 999
    "#;

    let mut parser = Parser::new(source).unwrap();
    let (ast, span_map) = parser.parse().unwrap();
    let result = compile_and_run(&ast, span_map);
    assert!(result.is_err(), "Float index assignment should be rejected");
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must be an integer"), "Error should mention integer requirement, got: {}", err_msg);
}

#[test]
fn test_float_index_in_expression() {
    let source = r#"
        let arr = [10, 20, 30]
        let idx = 2.5
        let val = arr[idx]
    "#;

    let mut parser = Parser::new(source).unwrap();
    let (ast, span_map) = parser.parse().unwrap();
    let result = compile_and_run(&ast, span_map);
    assert!(result.is_err(), "Variable float index should be rejected");
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must be an integer"), "Error should mention integer requirement, got: {}", err_msg);
}
