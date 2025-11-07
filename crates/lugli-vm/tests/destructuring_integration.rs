mod helpers;
use lugli_common::Value;

fn run_code(source: &str) -> Result<Value, String> {
    let (program, span_map) = lugli_parser::parse(source).map_err(|e| e.to_string())?;
    let mut vm = lugli_vm::Vm::new();
    vm.compile_and_run(&program, span_map).map_err(|e| e.to_string())
}

fn run_and_get_number(source: &str) -> Result<f64, String> {
    match run_code(source)? {
        Value::Number(n) => Ok(n),
        other => Err(format!("Expected number, got {:?}", other)),
    }
}

#[test]
fn test_list_destructuring_basic() {
    let source = r#"
        let [x, y] = [10, 20]
        x + y
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 30.0);
}

#[test]
fn test_list_destructuring_three_values() {
    let source = r#"
        let [a, b, c] = [1, 2, 3]
        a + b + c
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 6.0);
}

#[test]
fn test_multiple_assignment_shorthand() {
    let source = r#"
        let x, y = 100, 200
        x + y
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 300.0);
}

#[test]
fn test_dict_destructuring_basic() {
    let source = r#"
        let person = {"name": "Alice", "age": 30}
        let {name, age} = person
        age
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 30.0);
}

#[test]
fn test_dict_destructuring_with_renaming() {
    let source = r#"
        let point = {"x": 5, "y": 10}
        let {x: a, y: b} = point
        a + b
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 15.0);
}

#[test]
fn test_nested_list_destructuring() {
    let source = r#"
        let [x, [y, z]] = [1, [2, 3]]
        x + y + z
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 6.0);
}

#[test]
fn test_nested_dict_destructuring() {
    let source = r#"
        let data = {"outer": {"inner": 42}}
        let {outer: {inner: value}} = data
        value
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 42.0);
}

#[test]
fn test_for_loop_list_destructuring() {
    let source = r#"
        let pairs = [[1, 2], [3, 4], [5, 6]]
        mut sum = 0
        for [a, b] in pairs {
            sum = sum + a + b
        }
        sum
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 21.0); // 1+2+3+4+5+6
}

#[test]
fn test_for_loop_with_enumerate_destructuring() {
    let source = r#"
        let items = [10, 20, 30]
        mut total = 0
        for [index, value] in enumerate(items) {
            total = total + index + value
        }
        total
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 63.0); // (0+10) + (1+20) + (2+30)
}

#[test]
fn test_for_loop_dict_destructuring() {
    let source = r#"
        let items = [
            {"x": 1, "y": 2},
            {"x": 3, "y": 4}
        ]
        mut sum = 0
        for {x, y} in items {
            sum = sum + x + y
        }
        sum
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 10.0); // 1+2+3+4
}

#[test]
fn test_wildcard_destructuring() {
    let source = r#"
        let [x, _, z] = [1, 999, 3]
        x + z
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 4.0); // 999 is discarded
}

#[test]
fn test_mixed_destructuring() {
    let source = r#"
        let point = {"x": 5, "y": 10}
        let [a, b, p] = [1, 2, point]
        let {x, y} = p
        a + b + x + y
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 18.0); // 1+2+5+10
}

#[test]
fn test_destructuring_with_functions() {
    let source = r#"
        fn get_pair() {
            return [42, 84]
        }
        let [a, b] = get_pair()
        a + b
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 126.0);
}

#[test]
fn test_destructuring_in_closure() {
    let source = r#"
        let pairs = [[1, 2], [3, 4]]
        let add_pairs = fn(ps) {
            mut sum = 0
            for [a, b] in ps {
                sum = sum + a + b
            }
            return sum
        }
        add_pairs(pairs)
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 10.0);
}

#[test]
fn test_scoped_destructuring() {
    let source = r#"
        let x = 100
        let y = 200
        if true {
            let [x, y] = [1, 2]
            x + y
        }
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 3.0);
}

#[test]
fn test_mutable_destructuring() {
    let source = r#"
        mut [x, y] = [5, 10]
        x = x + 1
        y = y + 1
        x + y
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 17.0); // (5+1) + (10+1)
}

#[test]
fn test_const_destructuring() {
    let source = r#"
        const [PI, E] = [3.14, 2.71]
        PI + E
    "#;
    let result = run_and_get_number(source).unwrap();
    assert!((result - 5.85).abs() < 0.01);
}

#[test]
fn test_list_comprehension_with_destructuring() {
    let source = r#"
        let pairs = [[1, 2], [3, 4], [5, 6]]
        let sums = [a + b for [a, b] in pairs]
        sums[0] + sums[1] + sums[2]
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 21.0); // 3 + 7 + 11
}

#[test]
fn test_nested_comprehension_destructuring() {
    let source = r#"
        let coords = [[1, 2], [3, 4]]
        let values = [x + y for [x, y] in coords]
        values[0] + values[1]
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 10.0); // (1+2) + (3+4)
}

#[test]
fn test_swap_variables() {
    let source = r#"
        mut x = 10
        mut y = 20
        [x, y] = [y, x]
        x
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 20.0);
}

#[test]
fn test_fibonacci_with_destructuring() {
    let source = r#"
        mut a = 0
        mut b = 1
        mut i = 0
        while i < 10 {
            [a, b] = [b, a + b]
            i = i + 1
        }
        a
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 55.0);
}

#[test]
fn test_complex_data_processing() {
    let source = r#"
        let users = [
            {"name": "Alice", "score": 90},
            {"name": "Bob", "score": 85},
            {"name": "Charlie", "score": 95}
        ]

        mut total = 0
        for {name, score} in users {
            total = total + score
        }
        total / 3
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 90.0);
}

#[test]
fn test_matrix_destructuring() {
    let source = r#"
        let matrix = [
            [1, 2, 3],
            [4, 5, 6],
            [7, 8, 9]
        ]

        mut sum = 0
        for row in matrix {
            let [a, b, c] = row
            sum = sum + a + b + c
        }
        sum
    "#;
    let result = run_and_get_number(source).unwrap();
    assert_eq!(result, 45.0); // 1+2+...+9
}
