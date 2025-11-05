/// Tests for advanced expression features: Block, If, and Nested List Comprehensions
use lugli_common::Value;
use lugli_vm::compile_and_run;

fn run_and_get_number(source: &str) -> Result<f64, String> {
    let (program, span_map) = lugli_parser::parse(source).map_err(|e| e.to_string())?;
    let result = compile_and_run(&program, span_map).map_err(|e| e.to_string())?;
    match result {
        Value::Number(n) => Ok(n),
        other => Err(format!("Expected number, got {:?}", other)),
    }
}

fn run_code(source: &str) -> Result<Value, String> {
    let (program, span_map) = lugli_parser::parse(source).map_err(|e| e.to_string())?;
    compile_and_run(&program, span_map).map_err(|e| e.to_string())
}

mod block_expressions {
    use super::*;

    #[test]
    fn test_block_in_match_arm() {
        let source = r#"
            let x = 1
            match x {
                1 => {
                    let y = 10
                    let z = 20
                    y + z
                },
                _ => 0
            }
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 30.0);
    }

    #[test]
    fn test_nested_match_without_parens() {
        let source = r#"
            fn route(path, method) {
                match path {
                    "/users" => {
                        match method {
                            "GET" => 200,
                            "POST" => 201,
                            _ => 405
                        }
                    },
                    "/health" => 200,
                    _ => 404
                }
            }

            let r1 = route("/users", "GET")
            let r2 = route("/users", "POST")
            let r3 = route("/health", "GET")
            r1 + r2 + r3
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 601.0);
    }

    #[test]
    fn test_block_with_multiple_statements() {
        let source = r#"
            match 5 {
                5 => {
                    mut acc = 0
                    acc = acc + 10
                    acc = acc + 20
                    acc = acc + 5
                    acc
                },
                _ => 0
            }
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 35.0);
    }

    #[test]
    fn test_block_returns_last_expression() {
        let source = r#"
            match 1 {
                1 => {
                    let a = 5
                    let b = 10
                    a * b
                },
                _ => 0
            }
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 50.0);
    }

    #[test]
    fn test_block_returns_null_if_no_expression() {
        let source = r#"
            let result = match 1 {
                1 => {
                    mut x = 5
                    x = x + 10
                },
                _ => 0
            }
            result
        "#;
        let result = run_code(source).unwrap();
        assert_eq!(result, Value::Null);
    }
}

mod if_expressions {
    use super::*;

    #[test]
    fn test_if_expression_basic() {
        let source = r#"
            let x = if true { 10 } else { 20 }
            x
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 10.0);
    }

    #[test]
    fn test_if_expression_else_branch() {
        let source = r#"
            let x = if false { 10 } else { 20 }
            x
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 20.0);
    }

    #[test]
    fn test_if_expression_with_condition() {
        let source = r#"
            let a = 15
            let result = if a > 10 { 100 } else { 50 }
            result
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_if_expression_in_arithmetic() {
        let source = r#"
            let a = 5
            let result = (if a > 3 { 100 } else { 50 }) + 10
            result
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 110.0);
    }

    #[test]
    fn test_nested_if_expression() {
        let source = r#"
            let x = 5
            let result = if x > 10 {
                100
            } else {
                if x > 3 { 50 } else { 25 }
            }
            result
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 50.0);
    }

    #[test]
    fn test_if_expression_without_else_returns_null() {
        let source = r#"
            let result = if false { 10 }
            result
        "#;
        let result = run_code(source).unwrap();
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_if_expression_with_blocks() {
        let source = r#"
            let x = 5
            let result = if x > 3 {
                let a = 10
                let b = 20
                a + b
            } else {
                let c = 5
                c * 2
            }
            result
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 30.0);
    }

    #[test]
    fn test_if_expression_in_function() {
        let source = r#"
            fn abs(x) {
                if x < 0 { -x } else { x }
            }

            let r1 = abs(-5)
            let r2 = abs(10)
            r1 + r2
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 15.0);
    }
}

mod nested_list_comprehensions {
    use super::*;

    #[test]
    fn test_nested_comprehension_basic() {
        let source = r#"
            let result = [x*y for x in [1,2] for y in [3,4]]
            len(result)
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 4.0); // [3, 4, 6, 8]
    }

    #[test]
    fn test_nested_comprehension_values() {
        let source = r#"
            let result = [x*y for x in [2,3] for y in [10,20]]
            result[0] + result[1] + result[2] + result[3]
        "#;
        let result = run_and_get_number(source).unwrap();
        // x=2,y=10 -> 20
        // x=2,y=20 -> 40
        // x=3,y=10 -> 30
        // x=3,y=20 -> 60
        // Total: 150
        assert_eq!(result, 150.0);
    }

    #[test]
    fn test_triple_nested_comprehension() {
        let source = r#"
            let result = [x+y+z for x in [1,2] for y in [10,20] for z in [100]]
            len(result)
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 4.0); // 2 * 2 * 1 = 4 combinations
    }

    #[test]
    fn test_nested_comprehension_with_filter_first_clause() {
        let source = r#"
            let result = [x*y for x in [1,2,3,4] if x > 2 for y in [10,20]]
            len(result)
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 4.0); // x in [3,4], y in [10,20] -> 2*2 = 4
    }

    #[test]
    fn test_nested_comprehension_with_filter_second_clause() {
        let source = r#"
            let result = [x*y for x in [2,3] for y in [10,20,30] if y < 25]
            len(result)
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 4.0); // x in [2,3], y in [10,20] -> 2*2 = 4
    }

    #[test]
    fn test_nested_comprehension_with_filters_both() {
        let source = r#"
            let result = [x*y for x in range(1,5) if x > 1 for y in range(1,5) if y > 1]
            len(result)
        "#;
        let result = run_and_get_number(source).unwrap();
        // x in [2,3,4], y in [2,3,4] -> 3*3 = 9
        assert_eq!(result, 9.0);
    }

    #[test]
    fn test_nested_comprehension_multiplication_table() {
        let source = r#"
            let table = [x*y for x in range(1,4) for y in range(1,4)]
            # 1*1=1, 1*2=2, 1*3=3, 2*1=2, 2*2=4, 2*3=6, 3*1=3, 3*2=6, 3*3=9
            # Sum: 1+2+3+2+4+6+3+6+9 = 36
            mut sum = 0
            for val in table {
                sum = sum + val
            }
            sum
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 36.0);
    }

    #[test]
    fn test_nested_comprehension_with_range() {
        let source = r#"
            let pairs = [[x,y] for x in range(1,3) for y in range(1,3)]
            len(pairs)
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 4.0); // (1,1), (1,2), (2,1), (2,2)
    }
}

mod integration_tests {
    use super::*;

    #[test]
    fn test_if_expression_with_nested_comprehension() {
        let source = r#"
            let x = 5
            let result = if x > 3 {
                [a*b for a in [1,2] for b in [10,20]]
            } else {
                []
            }
            len(result)
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_match_with_blocks_and_if_expressions() {
        let source = r#"
            fn process(x) {
                match x {
                    1 => {
                        let val = if x > 0 { 100 } else { 50 }
                        val + 1
                    },
                    2 => {
                        let val = if x > 1 { 200 } else { 100 }
                        val + 2
                    },
                    _ => 0
                }
            }

            process(1) + process(2)
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 303.0); // 101 + 202
    }

    #[test]
    fn test_complex_nested_structures() {
        let source = r#"
            fn generate_data(n) {
                if n > 0 {
                    [x*2 for x in range(1, n+1)]
                } else {
                    []
                }
            }

            let data = generate_data(3)
            mut sum = 0
            for val in data {
                sum = sum + val
            }
            sum
        "#;
        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 12.0); // [2,4,6] -> 12
    }
}
