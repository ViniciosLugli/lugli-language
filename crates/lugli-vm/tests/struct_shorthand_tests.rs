mod helpers;

use helpers::assert_execution_succeeds;
use lugli_common::Value;

#[test]
fn test_basic_struct_shorthand() {
    let source = r#"
        struct Point { x, y }
        let x = 10
        let y = 20
        let p = Point { x, y }
        p.x + p.y
    "#;
    assert_execution_succeeds(source, Value::Number(30.0));
}

#[test]
fn test_mixed_shorthand_longhand() {
    let source = r#"
        struct Person { name, age, email }
        let name = "Alice"
        let age = 25
        let p = Person { name, age, email: "alice@example.com" }
        p.age
    "#;
    assert_execution_succeeds(source, Value::Number(25.0));
}

#[test]
fn test_shorthand_in_function() {
    let source = r#"
        struct Point { x, y }
        fn create_point(x, y) {
            return Point { x, y }
        }
        let p = create_point(5, 15)
        p.x + p.y
    "#;
    assert_execution_succeeds(source, Value::Number(20.0));
}

#[test]
fn test_nested_struct_shorthand() {
    let source = r#"
        struct Point { x, y }
        struct Rectangle { top_left, bottom_right }

        let top_left = Point { x: 0, y: 0 }
        let bottom_right = Point { x: 100, y: 50 }
        let rect = Rectangle { top_left, bottom_right }

        rect.bottom_right.x + rect.bottom_right.y
    "#;
    assert_execution_succeeds(source, Value::Number(150.0));
}

#[test]
fn test_shorthand_with_struct_declaration() {
    let source = r#"
        let width = 800
        let height = 600
        struct Size { width, height }
        let size = Size { width, height }
        size.width + size.height
    "#;
    assert_execution_succeeds(source, Value::Number(1400.0));
}

#[test]
fn test_shorthand_in_list() {
    let source = r#"
        struct Point { x, y }
        let x1 = 1
        let y1 = 2
        let x2 = 3
        let y2 = 4

        let points = [
            Point { x: x1, y: y1 },
            Point { x: x2, y: y2 }
        ]

        points[0].x + points[1].y
    "#;
    assert_execution_succeeds(source, Value::Number(5.0));
}

#[test]
fn test_all_longhand_still_works() {
    let source = r#"
        struct Point { x, y }
        let x = 10
        let y = 20
        let p = Point { x: x, y: y }
        p.x + p.y
    "#;
    assert_execution_succeeds(source, Value::Number(30.0));
}

#[test]
fn test_shorthand_with_computation() {
    let source = r#"
        struct Point { x, y }
        let a = 5
        let b = 10
        let x = a * 2
        let y = b + 5
        let p = Point { x, y }
        p.x + p.y
    "#;
    assert_execution_succeeds(source, Value::Number(25.0));
}
