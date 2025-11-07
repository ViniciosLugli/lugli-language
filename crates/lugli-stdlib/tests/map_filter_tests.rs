use lugli_common::{StringPool, Value};
use lugli_stdlib::core::list::{list_filter, list_map};
use std::{cell::RefCell, rc::Rc};

fn create_test_list(values: Vec<f64>) -> Value { Value::List(Rc::new(RefCell::new(values.into_iter().map(Value::Number).collect()))) }

fn create_test_fn() -> Value {
    Value::Function {
        name: Rc::from("double"),
        params: Rc::new(vec!["x".to_string()]),
        body_start: 0,
        bytecode_id: 0,
        required_count: 1,
        defaults: Rc::new(vec![]),
    }
}

#[test]
fn test_map_explains_limitation() {
    let mut pool = StringPool::new();

    let list = create_test_list(vec![1.0, 2.0, 3.0]);
    let func = create_test_fn();

    let result = list_map(&[list, func], &mut pool);

    assert!(result.is_err());
    let err = result.unwrap_err();

    let err_msg = format!("{}", err);
    assert!(err_msg.contains("cannot call user functions"));
    assert!(err_msg.contains("list comprehensions"));
    assert!(err_msg.contains("[transform(x) for x in list]"));
    assert!(err_msg.contains("Double values"));
}

#[test]
fn test_filter_explains_limitation() {
    let mut pool = StringPool::new();

    let list = create_test_list(vec![1.0, 2.0, 3.0, 4.0]);
    let func = create_test_fn();

    let result = list_filter(&[list, func], &mut pool);

    assert!(result.is_err());
    let err = result.unwrap_err();

    let err_msg = format!("{}", err);
    assert!(err_msg.contains("cannot call user functions"));
    assert!(err_msg.contains("list comprehensions"));
    assert!(err_msg.contains("[x for x in list if predicate(x)]"));
    assert!(err_msg.contains("Even numbers"));
}

#[test]
fn test_map_checks_arity() {
    let mut pool = StringPool::new();

    let result = list_map(&[], &mut pool);
    assert!(result.is_err());

    let list = create_test_list(vec![1.0, 2.0]);
    let result = list_map(&[list], &mut pool);
    assert!(result.is_err());
}

#[test]
fn test_filter_checks_arity() {
    let mut pool = StringPool::new();

    let result = list_filter(&[], &mut pool);
    assert!(result.is_err());

    let list = create_test_list(vec![1.0, 2.0]);
    let result = list_filter(&[list], &mut pool);
    assert!(result.is_err());
}

#[test]
fn test_map_with_closure() {
    let mut pool = StringPool::new();

    let list = create_test_list(vec![1.0, 2.0, 3.0]);
    let closure = Value::Closure {
        name: Rc::from("add_n"),
        params: Rc::new(vec!["x".to_string()]),
        body_start: 0,
        bytecode_id: 0,
        upvalues: vec![],
        required_count: 1,
        defaults: Rc::new(vec![]),
    };

    let result = list_map(&[list, closure], &mut pool);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("cannot call user functions"));
}

#[test]
fn test_filter_with_closure() {
    let mut pool = StringPool::new();

    let list = create_test_list(vec![1.0, 2.0, 3.0, 4.0]);
    let closure = Value::Closure {
        name: Rc::from("is_even"),
        params: Rc::new(vec!["x".to_string()]),
        body_start: 0,
        bytecode_id: 0,
        upvalues: vec![],
        required_count: 1,
        defaults: Rc::new(vec![]),
    };

    let result = list_filter(&[list, closure], &mut pool);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("cannot call user functions"));
}
