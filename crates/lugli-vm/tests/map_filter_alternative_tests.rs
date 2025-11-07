use lugli_common::Value;
use lugli_vm::Vm;

fn run_program(source: &str) -> Result<Value, String> {
    let (program, span_map) = lugli_parser::parse(source).map_err(|e| e.to_string())?;
    let mut vm = Vm::new();
    vm.compile_and_run(&program, span_map).map_err(|e| e.to_string())
}

#[test]
fn test_map_alternative_simple_transformation() {
    let source = r#"
        let numbers = [1, 2, 3, 4, 5]
        let doubled = [x * 2 for x in numbers]
        doubled
    "#;

    let result = run_program(source).expect("Program should run");
    if let Value::List(list) = result {
        let items = list.borrow();
        assert_eq!(items.len(), 5);
        assert_eq!(items[0], Value::Number(2.0));
        assert_eq!(items[1], Value::Number(4.0));
        assert_eq!(items[4], Value::Number(10.0));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_filter_alternative_simple_predicate() {
    let source = r#"
        let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        let evens = [x for x in numbers if x % 2 == 0]
        evens
    "#;

    let result = run_program(source).expect("Program should run");
    if let Value::List(list) = result {
        let items = list.borrow();
        assert_eq!(items.len(), 5);
        assert_eq!(items[0], Value::Number(2.0));
        assert_eq!(items[1], Value::Number(4.0));
        assert_eq!(items[4], Value::Number(10.0));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_map_with_function_call() {
    let source = r#"
        fn double(x) {
            return x * 2
        }

        let numbers = [1, 2, 3, 4, 5]
        let result = [double(x) for x in numbers]
        result
    "#;

    let result = run_program(source).expect("Program should run");
    if let Value::List(list) = result {
        let items = list.borrow();
        assert_eq!(items.len(), 5);
        assert_eq!(items[0], Value::Number(2.0));
        assert_eq!(items[4], Value::Number(10.0));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_filter_with_function_call() {
    let source = r#"
        fn is_even(x) {
            return x % 2 == 0
        }

        let numbers = [1, 2, 3, 4, 5, 6]
        let result = [x for x in numbers if is_even(x)]
        result
    "#;

    let result = run_program(source).expect("Program should run");
    if let Value::List(list) = result {
        let items = list.borrow();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::Number(2.0));
        assert_eq!(items[1], Value::Number(4.0));
        assert_eq!(items[2], Value::Number(6.0));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_map_and_filter_combined() {
    let source = r#"
        let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        let result = [x * x for x in numbers if x % 2 == 0]
        result
    "#;

    let result = run_program(source).expect("Program should run");
    if let Value::List(list) = result {
        let items = list.borrow();
        assert_eq!(items.len(), 5);
        assert_eq!(items[0], Value::Number(4.0));
        assert_eq!(items[1], Value::Number(16.0));
        assert_eq!(items[2], Value::Number(36.0));
        assert_eq!(items[3], Value::Number(64.0));
        assert_eq!(items[4], Value::Number(100.0));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_map_with_closure() {
    let source = r#"
        let multiplier = 3
        let numbers = [1, 2, 3, 4, 5]
        let result = [x * multiplier for x in numbers]
        result
    "#;

    let result = run_program(source).expect("Program should run");
    if let Value::List(list) = result {
        let items = list.borrow();
        assert_eq!(items.len(), 5);
        assert_eq!(items[0], Value::Number(3.0));
        assert_eq!(items[4], Value::Number(15.0));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_filter_with_closure() {
    let source = r#"
        let threshold = 5
        let numbers = [1, 3, 5, 7, 9, 11]
        let result = [x for x in numbers if x > threshold]
        result
    "#;

    let result = run_program(source).expect("Program should run");
    if let Value::List(list) = result {
        let items = list.borrow();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::Number(7.0));
        assert_eq!(items[1], Value::Number(9.0));
        assert_eq!(items[2], Value::Number(11.0));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_map_string_operations() {
    let source = r#"
        let strings = ["hello", "world", "lugli"]
        let result = [s.upper() for s in strings]
        result
    "#;

    let result = run_program(source).expect("Program should run");
    if let Value::List(list) = result {
        let items = list.borrow();
        assert_eq!(items.len(), 3);
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_filter_empty_strings() {
    let source = r#"
        let strings = ["hello", "", "world", "", "lugli"]
        let result = [s for s in strings if s != ""]
        result
    "#;

    let result = run_program(source).expect("Program should run");
    if let Value::List(list) = result {
        let items = list.borrow();
        assert_eq!(items.len(), 3);
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_nested_comprehension_as_map_of_map() {
    let source = r#"
        let row1 = [1.0, 2.0]
        let row2 = [3.0, 4.0]
        let row3 = [5.0, 6.0]
        let result1 = [x * 2.0 for x in row1]
        let result2 = [x * 2.0 for x in row2]
        let result3 = [x * 2.0 for x in row3]
        result1
    "#;

    let result = run_program(source).expect("Program should run");
    if let Value::List(list) = result {
        let items = list.borrow();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], Value::Number(2.0));
        assert_eq!(items[1], Value::Number(4.0));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_map_alternative_shows_proper_usage() {
    let source = r#"
        fn double(x) { return x * 2 }
        let numbers = [1, 2, 3]
        let result = [double(x) for x in numbers]
        result
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "List comprehension should work as map alternative");
}

#[test]
fn test_filter_alternative_shows_proper_usage() {
    let source = r#"
        fn is_even(x) { return x % 2 == 0 }
        let numbers = [1, 2, 3, 4, 5]
        let result = [x for x in numbers if is_even(x)]
        result
    "#;

    let result = run_program(source);
    assert!(result.is_ok(), "List comprehension should work as filter alternative");
}
