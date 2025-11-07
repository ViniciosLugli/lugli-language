use lugli_parser::Parser;
use lugli_vm::Vm;

fn run_code(code: &str) -> String {
    let mut parser = Parser::new(code).unwrap();
    let (ast, span_map) = parser.parse().unwrap();
    let bytecode = compile(&ast, span_map).unwrap();
    let result = run(&bytecode).unwrap();
    let pool = bytecode.string_pool.borrow();
    result.display_with_pool(&pool)
}

#[test]
fn test_fstring_simple() {
    let code = r#"
        let name = "Alice"
        let greeting = f"Hello, {name}!"
        greeting
    "#;
    assert_eq!(run_code(code), "Hello, Alice!");
}

#[test]
fn test_fstring_multiple_interpolations() {
    let code = r#"
        let name = "Bob"
        let age = 30
        let city = "NYC"
        let msg = f"{name} is {age} years old and lives in {city}"
        msg
    "#;
    assert_eq!(run_code(code), "Bob is 30 years old and lives in NYC");
}

#[test]
fn test_fstring_with_expressions() {
    let code = r#"
        let x = 10
        let y = 20
        let result = f"The sum of {x} and {y} is {x + y}"
        result
    "#;
    assert_eq!(run_code(code), "The sum of 10 and 20 is 30");
}

#[test]
fn test_fstring_with_method_calls() {
    let code = r#"
        let items = [1, 2, 3, 4, 5]
        let msg = f"List has {items.len()} items"
        msg
    "#;
    assert_eq!(run_code(code), "List has 5 items");
}

#[test]
fn test_fstring_empty() {
    let code = r#"
        let empty = f""
        empty
    "#;
    assert_eq!(run_code(code), "");
}

#[test]
fn test_fstring_text_only() {
    let code = r#"
        let msg = f"No interpolation here"
        msg
    "#;
    assert_eq!(run_code(code), "No interpolation here");
}

#[test]
fn test_fstring_single_interpolation() {
    let code = r#"
        let value = 42
        let msg = f"{value}"
        msg
    "#;
    assert_eq!(run_code(code), "42");
}

#[test]
fn test_fstring_with_boolean() {
    let code = r#"
        let flag = true
        let msg = f"Status: {flag}"
        msg
    "#;
    assert_eq!(run_code(code), "Status: true");
}

#[test]
fn test_fstring_with_null() {
    let code = r#"
        let value = null
        let msg = f"Value is {value}"
        msg
    "#;
    assert_eq!(run_code(code), "Value is null");
}

#[test]
fn test_fstring_with_list() {
    let code = r#"
        let nums = [1, 2, 3]
        let msg = f"Numbers: {nums}"
        msg
    "#;
    assert_eq!(run_code(code), "Numbers: [1, 2, 3]");
}

#[test]
fn test_fstring_with_dict() {
    let code = r#"
        let data = {"key": "value"}
        let msg = f"Data: {data}"
        msg
    "#;
    let result = run_code(code);
    assert!(result.contains("Data:"));
    assert!(result.contains("key"));
    assert!(result.contains("value"));
}

#[test]
fn test_fstring_with_nested_expressions() {
    let code = r#"
        let x = 5
        let msg = f"Result: {(x * 2) + 10}"
        msg
    "#;
    assert_eq!(run_code(code), "Result: 20");
}

#[test]
fn test_fstring_with_comparison() {
    let code = r#"
        let a = 10
        let b = 20
        let msg = f"{a} < {b} is {a < b}"
        msg
    "#;
    assert_eq!(run_code(code), "10 < 20 is true");
}

#[test]
fn test_fstring_in_function() {
    let code = r#"
        fn greet(name) {
            return f"Hello, {name}!"
        }
        greet("Charlie")
    "#;
    assert_eq!(run_code(code), "Hello, Charlie!");
}

#[test]
fn test_fstring_with_string_methods() {
    let code = r#"
        let text = "hello"
        let msg = f"Uppercase: {text.upper()}"
        msg
    "#;
    assert_eq!(run_code(code), "Uppercase: HELLO");
}

#[test]
fn test_fstring_concatenation() {
    let code = r#"
        let first = f"Hello"
        let second = f"World"
        let combined = first + " " + second
        combined
    "#;
    assert_eq!(run_code(code), "Hello World");
}

#[test]
fn test_fstring_in_struct() {
    let code = r#"
        struct Person {
            name: "Unknown"
            age: 0

            fn describe(self) {
                return f"{self.name} is {self.age} years old"
            }
        }

        let p = Person { name: "Dave", age: 25 }
        p.describe()
    "#;
    assert_eq!(run_code(code), "Dave is 25 years old");
}

#[test]
fn test_fstring_with_negative_numbers() {
    let code = r#"
        let temp = -5
        let msg = f"Temperature: {temp}°C"
        msg
    "#;
    assert_eq!(run_code(code), "Temperature: -5°C");
}

#[test]
fn test_fstring_with_floating_point() {
    let code = r#"
        let pi = 3.14159
        let msg = f"Pi is approximately {pi}"
        msg
    "#;
    assert_eq!(run_code(code), "Pi is approximately 3.14159");
}

#[test]
fn test_fstring_with_index_access() {
    let code = r#"
        let items = ["apple", "banana", "cherry"]
        let msg = f"First item: {items[0]}"
        msg
    "#;
    assert_eq!(run_code(code), "First item: apple");
}

#[test]
fn test_fstring_multiline_expression() {
    let code = r#"
        let x = 10
        let y = 20
        let z = 30
        let msg = f"Sum: {x + y + z}"
        msg
    "#;
    assert_eq!(run_code(code), "Sum: 60");
}

#[test]
fn test_fstring_with_conditional() {
    let code = r#"
        let score = 85
        let grade = match score {
            s if s >= 90 => "A",
            s if s >= 80 => "B",
            _ => "C"
        }
        let msg = f"Score {score} = Grade {grade}"
        msg
    "#;
    assert_eq!(run_code(code), "Score 85 = Grade B");
}

#[test]
fn test_fstring_in_loop() {
    let code = r#"
        let results = []
        for i in [1, 2, 3] {
            let msg = f"Item {i}"
            results.push(msg)
        }
        results[0]
    "#;
    assert_eq!(run_code(code), "Item 1");
}

#[test]
fn test_fstring_with_match_expression() {
    let code = r#"
        let value = 42
        let description = match value {
            42 => "answer",
            _ => "other"
        }
        let msg = f"The {value} is the {description}"
        msg
    "#;
    assert_eq!(run_code(code), "The 42 is the answer");
}
