use lugli_parser::parse;
use lugli_vm::Vm;

/// Helper function to run source code
fn run_test(source: &str) -> Result<lugli_common::Value, String> {
    let (program, span_map) = parse(source).map_err(|e| e.to_string())?;
    let mut vm = Vm::new();
    vm.compile_and_run(&program, span_map).map_err(|e| e.to_string())
}

#[test]
#[ignore] // Slow test - run with --ignored
fn test_large_list_operations() {
    let source = r#"
        fn test_large_list() {
            let large_list = []
            mut i = 0
            while i < 10000 {
                large_list.push(i)
                i = i + 1
            }

            mut sum = 0
            mut j = 0
            while j < large_list.len() {
                sum = sum + large_list[j]
                j = j + 1
            }
            return sum
        }
        test_large_list()
    "#;

    let result = run_test(source);
    assert!(result.is_ok());
    // Sum of 0..9999 = 49995000
    if let Ok(lugli_common::Value::Number(n)) = result {
        assert_eq!(n, 49995000.0);
    }
}

#[test]
#[ignore] // Slow test - run with --ignored
fn test_deep_nesting() {
    let source = r#"
        fn test_deep_nesting() {
            let root = { "level": 0, "child": null }
            let current = root

            mut i = 0
            while i < 100 {
                current["child"] = { "level": i + 1, "child": null }
                current = current["child"]
                i = i + 1
            }

            # Traverse back up
            mut depth = 0
            let node = root
            while node != null {
                depth = depth + 1
                node = node["child"]
            }

            return depth
        }
        test_deep_nesting()
    "#;

    let result = run_test(source);
    assert!(result.is_ok());
    if let Ok(lugli_common::Value::Number(n)) = result {
        assert_eq!(n, 101.0); // root + 100 children
    }
}

#[test]
#[ignore] // Slow test - run with --ignored
fn test_many_closures() {
    let source = r#"
        fn test_many_closures() {
            let closures = []

            mut i = 0
            while i < 1000 {
                let x = i
                let closure = fn() {
                    return x * 2
                }
                closures.push(closure)
                i = i + 1
            }

            mut sum = 0
            mut j = 0
            while j < closures.len() {
                sum = sum + closures[j]()
                j = j + 1
            }

            return sum
        }
        test_many_closures()
    "#;

    let result = run_test(source);
    assert!(result.is_ok());
    // Sum of (0..999) * 2 = 999000
    if let Ok(lugli_common::Value::Number(n)) = result {
        assert_eq!(n, 999000.0);
    }
}

#[test]
#[ignore] // Slow test - run with --ignored
fn test_heavy_string_operations() {
    let source = r#"
        fn test_string_ops() {
            let strings = []

            mut i = 0
            while i < 1000 {
                strings.push("string_")
                i = i + 1
            }

            let result = ""
            mut j = 0
            while j < 100 {
                result = result + strings[j]
                j = j + 1
            }

            return result.len()
        }
        test_string_ops()
    "#;

    let result = run_test(source);
    assert!(result.is_ok());
    // "string_" is 7 chars * 100 repetitions = 700
    if let Ok(lugli_common::Value::Number(n)) = result {
        assert_eq!(n, 700.0);
    }
}

#[test]
#[ignore] // Slow test - run with --ignored
fn test_large_dictionary() {
    let source = r#"
        fn test_large_dict() {
            let large_dict = {}

            mut i = 0
            while i < 5000 {
                large_dict[i] = i * 3
                i = i + 1
            }

            mut sum = 0
            mut j = 0
            while j < 5000 {
                sum = sum + large_dict[j]
                j = j + 1
            }

            return sum
        }
        test_large_dict()
    "#;

    let result = run_test(source);
    assert!(result.is_ok());
    // Sum of (0..4999) * 3 = 37492500
    if let Ok(lugli_common::Value::Number(n)) = result {
        assert_eq!(n, 37492500.0);
    }
}

#[test]
#[ignore] // Slow test - run with --ignored
fn test_nested_loops_stress() {
    let source = r#"
        fn test_nested_loops() {
            mut count = 0

            mut i = 0
            while i < 100 {
                mut j = 0
                while j < 100 {
                    count = count + 1
                    j = j + 1
                }
                i = i + 1
            }

            return count
        }
        test_nested_loops()
    "#;

    let result = run_test(source);
    assert!(result.is_ok());
    if let Ok(lugli_common::Value::Number(n)) = result {
        assert_eq!(n, 10000.0); // 100 * 100
    }
}

#[test]
#[ignore] // Slow test - run with --ignored
fn test_recursive_fibonacci_stress() {
    let source = r#"
        fn fib(n) {
            if n <= 1 {
                return n
            }
            return fib(n - 1) + fib(n - 2)
        }

        fib(25)
    "#;

    let result = run_test(source);
    assert!(result.is_ok());
    if let Ok(lugli_common::Value::Number(n)) = result {
        assert_eq!(n, 75025.0); // fib(25) = 75025
    }
}

#[test]
#[ignore] // Slow test - run with --ignored
fn test_list_comprehension_stress() {
    let source = r#"
        fn test_comprehensions() {
            # Generate large list
            let base = []
            mut i = 0
            while i < 5000 {
                base.push(i)
                i = i + 1
            }

            # Multiple transformations
            let doubled = [x * 2 for x in base]
            let filtered = [x for x in doubled if x % 3 == 0]

            return filtered.len()
        }
        test_comprehensions()
    "#;

    let result = run_test(source);
    assert!(result.is_ok());
    assert!(matches!(result, Ok(lugli_common::Value::Number(_))));
}

#[test]
#[ignore] // Slow test - run with --ignored
fn test_pattern_matching_stress() {
    let source = r#"
        fn classify(n) {
            match n {
                0 => "zero",
                1 => "one",
                2 => "two",
                n if n < 10 => "small",
                n if n < 100 => "medium",
                n if n < 1000 => "large",
                _ => "huge"
            }
        }

        fn test_matching() {
            mut count = 0
            mut i = 0
            while i < 10000 {
                let result = classify(i)
                if result != null {
                    count = count + 1
                }
                i = i + 1
            }
            return count
        }
        test_matching()
    "#;

    let result = run_test(source);
    assert!(result.is_ok());
    if let Ok(lugli_common::Value::Number(n)) = result {
        assert_eq!(n, 10000.0);
    }
}

#[test]
#[ignore] // Slow test - run with --ignored
fn test_struct_instantiation_stress() {
    let source = r#"
        struct Point {
            x
            y

            fn distance_from_origin(self) {
                return (self.x * self.x + self.y * self.y) ** 0.5
            }
        }

        fn test_structs() {
            let points = []

            mut i = 0
            while i < 2000 {
                let p = Point { x: i, y: i * 2 }
                points.push(p)
                i = i + 1
            }

            mut sum = 0.0
            mut j = 0
            while j < points.len() {
                sum = sum + points[j].distance_from_origin()
                j = j + 1
            }

            return sum > 0
        }
        test_structs()
    "#;

    let result = run_test(source);
    assert!(result.is_ok());
    if let Ok(lugli_common::Value::Bool(b)) = result {
        assert!(b);
    }
}

#[test]
fn test_small_list_operations() {
    let source = r#"
        let list = []
        mut i = 0
        while i < 100 {
            list.push(i)
            i = i + 1
        }

        mut sum = 0
        for x in list {
            sum = sum + x
        }
        sum
    "#;

    let result = run_test(source);
    assert!(result.is_ok());
    // Sum of 0..99 = 4950
    if let Ok(lugli_common::Value::Number(n)) = result {
        assert_eq!(n, 4950.0);
    }
}

#[test]
fn test_moderate_recursion() {
    let source = r#"
        fn factorial(n) {
            if n <= 1 {
                return 1
            }
            return n * factorial(n - 1)
        }

        factorial(10)
    "#;

    let result = run_test(source);
    assert!(result.is_ok());
    if let Ok(lugli_common::Value::Number(n)) = result {
        assert_eq!(n, 3628800.0); // 10! = 3628800
    }
}

#[test]
fn test_moderate_string_building() {
    let source = r#"
        let parts = []
        mut i = 0
        while i < 50 {
            parts.push("x")
            i = i + 1
        }

        let result = ""
        for p in parts {
            result = result + p
        }
        result.len()
    "#;

    let result = run_test(source);
    assert!(result.is_ok());
    if let Ok(lugli_common::Value::Number(n)) = result {
        assert_eq!(n, 50.0);
    }
}
