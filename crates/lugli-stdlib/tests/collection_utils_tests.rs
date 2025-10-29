use lugli_common::Value;
use lugli_stdlib::get_global_functions;
use std::{cell::RefCell, rc::Rc};

fn get_function(name: &str) -> Option<fn(&[Value]) -> Result<Value, lugli_common::LugliError>> {
    get_global_functions().into_iter().find(|(n, _)| *n == name).map(|(_, f)| f)
}

#[test]
fn test_zip_two_lists() {
    let zip = get_function("zip").unwrap();

    let list1 = Value::List(Rc::new(RefCell::new(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)])));

    let list2 =
        Value::List(Rc::new(RefCell::new(vec![Value::String("a".to_string()), Value::String("b".to_string()), Value::String("c".to_string())])));

    let result = zip(&[list1, list2]).unwrap();

    match result {
        Value::List(l) => {
            let items = l.borrow();
            assert_eq!(items.len(), 3);

            // Check first tuple
            if let Value::List(tuple) = &items[0] {
                let t = tuple.borrow();
                assert_eq!(t.len(), 2);
                assert!(matches!(t[0], Value::Number(1.0)));
                assert_eq!(t[1], Value::String("a".to_string()));
            } else {
                panic!("Expected tuple");
            }
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_zip_different_lengths() {
    let zip = get_function("zip").unwrap();

    let list1 = Value::List(Rc::new(RefCell::new(vec![Value::Number(1.0), Value::Number(2.0)])));

    let list2 =
        Value::List(Rc::new(RefCell::new(vec![Value::String("a".to_string()), Value::String("b".to_string()), Value::String("c".to_string())])));

    let result = zip(&[list1, list2]).unwrap();

    match result {
        Value::List(l) => {
            assert_eq!(l.borrow().len(), 2); // Min length
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_all_true() {
    let all = get_function("all").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Bool(true), Value::Number(1.0), Value::String("hello".to_string())])));

    let result = all(&[list]).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_all_false() {
    let all = get_function("all").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![
        Value::Bool(true),
        Value::Number(0.0), // Falsy
        Value::String("hello".to_string()),
    ])));

    let result = all(&[list]).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_any_true() {
    let any = get_function("any").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![
        Value::Bool(false),
        Value::Number(0.0),
        Value::Number(1.0), // Truthy
    ])));

    let result = any(&[list]).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_any_false() {
    let any = get_function("any").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Bool(false), Value::Number(0.0), Value::Null])));

    let result = any(&[list]).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_sum_basic() {
    let sum = get_function("sum").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0), Value::Number(4.0)])));

    let result = sum(&[list]).unwrap();
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn test_sum_with_start() {
    let sum = get_function("sum").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)])));

    let result = sum(&[list, Value::Number(10.0)]).unwrap();
    assert_eq!(result, Value::Number(16.0));
}

#[test]
fn test_min_numbers() {
    let min = get_function("min").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Number(5.0), Value::Number(2.0), Value::Number(8.0), Value::Number(1.0)])));

    let result = min(&[list]).unwrap();
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_min_strings() {
    let min = get_function("min").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![
        Value::String("zebra".to_string()),
        Value::String("apple".to_string()),
        Value::String("banana".to_string()),
    ])));

    let result = min(&[list]).unwrap();
    assert_eq!(result, Value::String("apple".to_string()));
}

#[test]
fn test_max_numbers() {
    let max = get_function("max").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Number(5.0), Value::Number(2.0), Value::Number(8.0), Value::Number(1.0)])));

    let result = max(&[list]).unwrap();
    assert_eq!(result, Value::Number(8.0));
}

#[test]
fn test_sorted_numbers() {
    let sorted = get_function("sorted").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Number(5.0), Value::Number(2.0), Value::Number(8.0), Value::Number(1.0)])));

    let result = sorted(&[list]).unwrap();

    match result {
        Value::List(l) => {
            let items = l.borrow();
            assert_eq!(items[0], Value::Number(1.0));
            assert_eq!(items[1], Value::Number(2.0));
            assert_eq!(items[2], Value::Number(5.0));
            assert_eq!(items[3], Value::Number(8.0));
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_sorted_reverse() {
    let sorted = get_function("sorted").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Number(5.0), Value::Number(2.0), Value::Number(8.0)])));

    let result = sorted(&[list, Value::Bool(true)]).unwrap();

    match result {
        Value::List(l) => {
            let items = l.borrow();
            assert_eq!(items[0], Value::Number(8.0));
            assert_eq!(items[1], Value::Number(5.0));
            assert_eq!(items[2], Value::Number(2.0));
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_reversed() {
    let reversed = get_function("reversed").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)])));

    let result = reversed(&[list]).unwrap();

    match result {
        Value::List(l) => {
            let items = l.borrow();
            assert_eq!(items[0], Value::Number(3.0));
            assert_eq!(items[1], Value::Number(2.0));
            assert_eq!(items[2], Value::Number(1.0));
        }
        _ => panic!("Expected list"),
    }
}
