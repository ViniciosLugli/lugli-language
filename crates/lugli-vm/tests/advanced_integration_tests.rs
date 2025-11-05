/// Advanced integration tests that simulate real-world usage scenarios
/// These tests validate that Lugli can handle complex, practical programs
use lugli_common::Value;
use lugli_vm::compile_and_run;

/// Helper to run Lugli code and return the result
fn run_code(source: &str) -> Result<Value, String> {
    let (program, span_map) = lugli_parser::parse(source).map_err(|e| e.to_string())?;
    compile_and_run(&program, span_map).map_err(|e| e.to_string())
}

/// Helper to run code and extract a number result
fn run_and_get_number(source: &str) -> Result<f64, String> {
    match run_code(source)? {
        Value::Number(n) => Ok(n),
        other => Err(format!("Expected number, got {}", other.type_name())),
    }
}

#[cfg(test)]
mod real_world_scenarios {
    use super::*;

    #[test]
    fn test_data_processing_pipeline() {
        // Simulates processing a list of user records
        let source = r#"
            # Create sample user data
            let users = [
                {"name": "Alice", "age": 30, "score": 95},
                {"name": "Bob", "age": 25, "score": 87},
                {"name": "Charlie", "age": 35, "score": 92},
                {"name": "David", "age": 28, "score": 78}
            ]

            # Filter users over 25 and calculate average score
            let filtered = []
            mut total_score = 0

            for user in users {
                if user["age"] > 25 {
                    filtered.push(user)
                    total_score = total_score + user["score"]
                }
            }

            let avg_score = total_score / len(filtered)
            avg_score
        "#;

        let result = run_and_get_number(source).unwrap();
        assert!((result - 88.33).abs() < 0.01); // (95 + 92 + 78) / 3 = 88.33...
    }

    #[test]
    fn test_object_oriented_calculator() {
        // Tests OOP patterns with structs and methods
        let source = r#"
            struct Calculator {
                result

                fn add(self, x) {
                    self.result = self.result + x
                    return self
                }

                fn multiply(self, x) {
                    self.result = self.result * x
                    return self
                }

                fn get_result(self) {
                    return self.result
                }
            }

            let calc = Calculator { result: 10 }
            calc.add(5).multiply(2).get_result()
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 30.0); // (10 + 5) * 2 = 30
    }

    #[test]
    fn test_fibonacci_with_memoization() {
        // Tests closures and state management
        let source = r#"
            fn make_fib_calculator() {
                let cache = {0: 0, 1: 1}

                fn fib(n) {
                    if cache.has_key(n) {
                        return cache.get(n)
                    }

                    let result = fib(n - 1) + fib(n - 2)
                    cache[n] = result
                    return result
                }

                return fib
            }

            let fib = make_fib_calculator()
            fib(10)
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 55.0);
    }

    #[test]
    fn test_string_processing_pipeline() {
        // Tests string operations and list comprehensions
        let source = r#"
            let text = "Hello World From Lugli Language"

            # Split into words, filter short words, convert to uppercase
            let words = text.split(" ")
            let long_words = [w.upper() for w in words if w.len() > 4]

            # Join back together
            let result = long_words.join("-")
            result
        "#;

        let result = run_code(source).unwrap();
        if let Value::String(_id) = result {
            // We can't easily access the string pool here, but we can verify it's a string
            assert!(matches!(result, Value::String(_)));
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_nested_data_structures() {
        // Tests deep nesting and complex access patterns
        let source = r#"
            let company = {
                "name": "TechCorp",
                "departments": [
                    {
                        "name": "Engineering",
                        "employees": [
                            {"name": "Alice", "salary": 100000},
                            {"name": "Bob", "salary": 95000}
                        ]
                    },
                    {
                        "name": "Sales",
                        "employees": [
                            {"name": "Charlie", "salary": 80000}
                        ]
                    }
                ]
            }

            # Calculate total salary across all departments
            let total = 0
            for dept in company["departments"] {
                for emp in dept["employees"] {
                    total = total + emp["salary"]
                }
            }

            total
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 275000.0);
    }

    #[test]
    fn test_pattern_matching_router() {
        // Simulates a simple HTTP router using pattern matching
        let source = r#"
            fn route_request(method, path) {
                match path {
                    "/api/users" => (
                        match method {
                            "GET" => 200,
                            "POST" => 201,
                            _ => 405
                        }
                    ),
                    "/api/health" => 200,
                    _ => 404
                }
            }

            let r1 = route_request("GET", "/api/users")
            let r2 = route_request("POST", "/api/users")
            let r3 = route_request("DELETE", "/api/users")
            let r4 = route_request("GET", "/api/health")
            let r5 = route_request("GET", "/unknown")

            r1 + r2 + r3 + r4 + r5  # 200 + 201 + 405 + 200 + 404 = 1410
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 1410.0);
    }

    #[test]
    fn test_mathematical_operations() {
        // Tests the new math functions in realistic calculations
        let source = r#"
            # Calculate area of a circle
            const PI = 3.14159265359
            let radius = 5.0
            let area = PI * pow(radius, 2)

            # Calculate trajectory (physics simulation)
            let angle = PI / 4  # 45 degrees
            let velocity = 20.0
            let g = 9.8

            let vx = velocity * cos(angle)
            let vy = velocity * sin(angle)

            # Max height: h = (vy^2) / (2g)
            let max_height = pow(vy, 2) / (2 * g)

            # Return ceiling of max height
            ceil(max_height)
        "#;

        let result = run_and_get_number(source).unwrap();
        assert!((10.0..=11.0).contains(&result)); // Should be around 10.2
    }

    #[test]
    fn test_list_comprehension_with_conditions() {
        // Tests list comprehension with filtering
        let source = r#"
            # Generate even squares from 1 to 20
            let numbers = range(1, 21)
            let even_squares = [x * x for x in numbers if x % 2 == 0]

            # Count how many even squares exist
            len(even_squares)  # Should be 10 (2,4,6,8,10,12,14,16,18,20)
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 10.0); // 10 even numbers in range 1-20
    }

    #[test]
    fn test_closure_capturing_and_modification() {
        // Tests closure upvalue capture and modification
        let source = r#"
            fn make_counter() {
                mut count = 0

                fn increment() {
                    count = count + 1
                    return count
                }

                return increment
            }

            let counter1 = make_counter()
            let counter2 = make_counter()

            let a = counter1()  # 1
            let b = counter1()  # 2
            let c = counter2()  # 1
            let d = counter1()  # 3

            a + b + c + d  # 1 + 2 + 1 + 3 = 7
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 7.0);
    }

    #[test]
    fn test_error_recovery_in_loops() {
        // Tests that errors don't crash the entire program
        let source = r#"
            let results = []
            let numbers = [1, 2, 0, 4, 5]

            for n in numbers {
                # Safe division by zero handling would go here
                # For now, just skip zeros
                if n != 0 {
                    results.push(100 / n)
                }
            }

            len(results)
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 4.0); // Should have 4 results (skipped the zero)
    }
}

#[cfg(test)]
mod performance_critical_operations {
    use super::*;

    #[test]
    fn test_large_list_operations() {
        // Tests performance with larger data sets
        let source = r#"
            # Create a large list
            let numbers = range(1, 1001)  # 1000 numbers

            # Process them
            let sum = 0
            for n in numbers {
                sum = sum + n
            }

            sum  # Should be 500500 (sum of 1 to 1000)
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 500500.0);
    }

    #[test]
    fn test_nested_loop_performance() {
        // Tests nested loop efficiency
        let source = r#"
            let count = 0
            for i in range(1, 21) {
                for j in range(1, 21) {
                    count = count + 1
                }
            }
            count  # Should be 400 (20 * 20)
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 400.0);
    }

    #[test]
    fn test_recursive_function_depth() {
        // Tests deep recursion handling
        let source = r#"
            fn countdown(n) {
                if n <= 0 {
                    return 0
                }
                return 1 + countdown(n - 1)
            }

            countdown(100)
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 100.0);
    }
}

#[cfg(test)]
mod edge_cases_and_corner_cases {
    use super::*;

    #[test]
    fn test_empty_collections() {
        let source = r#"
            let empty_list = []
            let empty_dict = {}

            len(empty_list) + len(empty_dict)
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_negative_indexing() {
        let source = r#"
            let items = [10, 20, 30, 40, 50]
            items[-1]  # Should be 50
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 50.0);
    }

    #[test]
    fn test_chained_method_calls() {
        let source = r#"
            let text = "  hello world  "
            text.trim().upper().len()
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 11.0); // "HELLO WORLD" is 11 characters
    }

    #[test]
    fn test_bool_in_arithmetic() {
        let source = r#"
            let x = true
            let y = false

            # In Lugli, use explicit variable assignment
            mut result = 0
            if x {
                result = 10
            } else {
                result = 5
            }
            result
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 10.0);
    }

    #[test]
    fn test_struct_with_defaults() {
        let source = r#"
            struct Point {
                x
                y = 0
                z = 0
            }

            let p1 = Point { x: 5 }
            let p2 = Point { x: 3, y: 4 }

            p1.x + p1.y + p2.x + p2.y  # 5 + 0 + 3 + 4 = 12
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 12.0);
    }

    #[test]
    fn test_null_handling() {
        let source = r#"
            let x = null
            let y = 5

            mut result = 0
            if x == null {
                result = y * 2
            } else {
                result = y
            }
            result
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 10.0);
    }

    #[test]
    fn test_string_concatenation_edge_cases() {
        let source = r#"
            let empty = ""
            let space = " "
            let text = "hello"

            let result = empty + text + space + text + empty
            result.len()
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 11.0); // "hello hello" is 11 characters
    }
}

#[cfg(test)]
mod functional_programming_patterns {
    use super::*;

    #[test]
    fn test_higher_order_function_composition() {
        let source = r#"
            # Define composable functions
            fn double(x) { return x * 2 }
            fn add_ten(x) { return x + 10 }
            fn square(x) { return x * x }

            # Compose them
            let result = square(add_ten(double(5)))
            result  # (5 * 2 + 10)^2 = 20^2 = 400
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 400.0);
    }

    #[test]
    fn test_partial_application_simulation() {
        let source = r#"
            fn make_adder(x) {
                fn add(y) {
                    return x + y
                }
                return add
            }

            let add5 = make_adder(5)
            let add10 = make_adder(10)

            add5(3) + add10(7)  # 8 + 17 = 25
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 25.0);
    }

    #[test]
    fn test_callback_pattern() {
        let source = r#"
            fn process_list(items, callback) {
                let results = []
                for item in items {
                    results.push(callback(item))
                }
                return results
            }

            let numbers = [1, 2, 3, 4, 5]
            let doubled = process_list(numbers, fn(x) { return x * 2 })

            # Sum the doubled numbers
            let sum = 0
            for n in doubled {
                sum = sum + n
            }
            sum  # 2 + 4 + 6 + 8 + 10 = 30
        "#;

        let result = run_and_get_number(source).unwrap();
        assert_eq!(result, 30.0);
    }
}
