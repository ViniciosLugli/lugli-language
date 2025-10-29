use lugli_common::{LugliError, StringPool, Value};
use lugli_stdlib::get_global_functions;
use std::{cell::RefCell, rc::Rc};

type NativeFunction = fn(&[Value], &mut StringPool) -> Result<Value, LugliError>;

fn get_function(name: &str) -> Option<NativeFunction> { get_global_functions().into_iter().find(|(n, _)| *n == name).map(|(_, f)| f) }

#[test]
fn test_zip_two_lists() {
    let mut pool = StringPool::new();
    let zip = get_function("zip").unwrap();

    let list1 = Value::List(Rc::new(RefCell::new(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)])));

    let list2 =
        Value::List(Rc::new(RefCell::new(vec![Value::String(pool.intern("a")), Value::String(pool.intern("b")), Value::String(pool.intern("c"))])));

    let result = zip(&[list1, list2], &mut pool).unwrap();

    match result {
        Value::List(l) => {
            let items = l.borrow();
            assert_eq!(items.len(), 3);

            if let Value::List(tuple) = &items[0] {
                let t = tuple.borrow();
                assert_eq!(t.len(), 2);
                assert!(matches!(t[0], Value::Number(1.0)));
                assert_eq!(t[1], Value::String(pool.intern("a")));
            } else {
                panic!("Expected tuple");
            }
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_zip_different_lengths() {
    let mut pool = StringPool::new();
    let zip = get_function("zip").unwrap();

    let list1 = Value::List(Rc::new(RefCell::new(vec![Value::Number(1.0), Value::Number(2.0)])));

    let list2 =
        Value::List(Rc::new(RefCell::new(vec![Value::String(pool.intern("a")), Value::String(pool.intern("b")), Value::String(pool.intern("c"))])));

    let result = zip(&[list1, list2], &mut pool).unwrap();

    match result {
        Value::List(l) => {
            assert_eq!(l.borrow().len(), 2);
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_all_true() {
    let mut pool = StringPool::new();
    let all = get_function("all").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Bool(true), Value::Number(1.0), Value::String(pool.intern("hello"))])));

    let result = all(&[list], &mut pool).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_all_false() {
    let mut pool = StringPool::new();
    let all = get_function("all").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Bool(true), Value::Number(0.0), Value::String(pool.intern("hello"))])));

    let result = all(&[list], &mut pool).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_any_true() {
    let mut pool = StringPool::new();
    let any = get_function("any").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Bool(false), Value::Number(0.0), Value::Number(1.0)])));

    let result = any(&[list], &mut pool).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_any_false() {
    let mut pool = StringPool::new();
    let any = get_function("any").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Bool(false), Value::Number(0.0), Value::Null])));

    let result = any(&[list], &mut pool).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_sum_basic() {
    let mut pool = StringPool::new();
    let sum = get_function("sum").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0), Value::Number(4.0)])));

    let result = sum(&[list], &mut pool).unwrap();
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn test_sum_with_start() {
    let mut pool = StringPool::new();
    let sum = get_function("sum").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)])));

    let result = sum(&[list, Value::Number(10.0)], &mut pool).unwrap();
    assert_eq!(result, Value::Number(16.0));
}

#[test]
fn test_min_numbers() {
    let mut pool = StringPool::new();
    let min = get_function("min").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Number(5.0), Value::Number(2.0), Value::Number(8.0), Value::Number(1.0)])));

    let result = min(&[list], &mut pool).unwrap();
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_min_strings() {
    let mut pool = StringPool::new();
    let min = get_function("min").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![
        Value::String(pool.intern("zebra")),
        Value::String(pool.intern("apple")),
        Value::String(pool.intern("banana")),
    ])));

    let result = min(&[list], &mut pool).unwrap();
    if let Value::String(id) = result {
        assert_eq!(pool.resolve(id), "apple");
    } else {
        panic!("Expected String result");
    }
}

#[test]
fn test_max_numbers() {
    let mut pool = StringPool::new();
    let max = get_function("max").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Number(5.0), Value::Number(2.0), Value::Number(8.0), Value::Number(1.0)])));

    let result = max(&[list], &mut pool).unwrap();
    assert_eq!(result, Value::Number(8.0));
}

#[test]
fn test_sorted_numbers() {
    let mut pool = StringPool::new();
    let sorted = get_function("sorted").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Number(5.0), Value::Number(2.0), Value::Number(8.0), Value::Number(1.0)])));

    let result = sorted(&[list], &mut pool).unwrap();

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
    let mut pool = StringPool::new();
    let sorted = get_function("sorted").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Number(5.0), Value::Number(2.0), Value::Number(8.0)])));

    let result = sorted(&[list, Value::Bool(true)], &mut pool).unwrap();

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
    let mut pool = StringPool::new();
    let reversed = get_function("reversed").unwrap();

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)])));

    let result = reversed(&[list], &mut pool).unwrap();

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
