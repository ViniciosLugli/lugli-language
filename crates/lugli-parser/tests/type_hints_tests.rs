use lugli_parser::Parser;

#[test]
fn test_variable_type_hints() {
    let source = r#"
        let x: num = 42
        let name: str = "Alice"
        let active: bool = true
        mut counter: num = 0
        const PI: f64 = 3.14159
    "#;

    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse();
    assert!(ast.is_ok(), "Parser should handle variable type hints: {:?}", ast.err());
}

#[test]
fn test_function_parameter_type_hints() {
    let source = r#"
        fn add(a: num, b: num) {
            return a + b
        }

        fn greet(name: str, age: num) {
            print(f"Hello {name}, you are {age}")
        }
    "#;

    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse();
    assert!(ast.is_ok(), "Parser should handle function parameter type hints: {:?}", ast.err());
}

#[test]
fn test_function_return_type_hints() {
    let source = r#"
        fn add(a: num, b: num) -> num {
            return a + b
        }

        fn get_name() -> str {
            return "Alice"
        }

        fn is_valid(x: num) -> bool {
            return x > 0
        }
    "#;

    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse();
    assert!(ast.is_ok(), "Parser should handle function return type hints: {:?}", ast.err());
}

#[test]
fn test_struct_field_type_hints() {
    let source = r#"
        struct Point {
            x: num
            y: num
        }

        struct Person {
            name: str
            age: num
            active: bool
        }
    "#;

    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse();
    assert!(ast.is_ok(), "Parser should handle struct field type hints: {:?}", ast.err());
}

#[test]
fn test_struct_field_type_hints_with_defaults() {
    let source = r#"
        struct Point {
            x: num = 0
            y: num = 0
        }

        struct Config {
            host: str = "localhost"
            port: num = 8080
            debug: bool = false
        }
    "#;

    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse();
    assert!(ast.is_ok(), "Parser should handle struct field type hints with defaults: {:?}", ast.err());
}

#[test]
fn test_mixed_type_hints_and_no_hints() {
    let source = r#"
        let x = 42
        let y: num = 100

        fn process(value, count: num) -> num {
            return value + count
        }

        struct Data {
            field1
            field2: str
            field3: num = 42
        }
    "#;

    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse();
    assert!(ast.is_ok(), "Parser should handle mixed type hints and no hints: {:?}", ast.err());
}

#[test]
fn test_complex_type_hints() {
    let source = r#"
        let items: List = [1, 2, 3]
        let data: Dict = {"a": 1, "b": 2}

        fn get_users() -> List {
            return []
        }

        struct Container {
            values: List = []
            mapping: Dict
        }
    "#;

    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse();
    assert!(ast.is_ok(), "Parser should handle complex type hints: {:?}", ast.err());
}

#[test]
fn test_type_hints_with_method_suffixes() {
    let source = r#"
        struct Counter {
            value: num

            fn increment!(self) -> num {
                self.value = self.value + 1
                return self.value
            }

            fn is_zero?(self) -> bool {
                return self.value == 0
            }
        }
    "#;

    let mut parser = Parser::new(source).unwrap();
    let ast = parser.parse();
    assert!(ast.is_ok(), "Parser should handle type hints with method suffixes: {:?}", ast.err());
}
