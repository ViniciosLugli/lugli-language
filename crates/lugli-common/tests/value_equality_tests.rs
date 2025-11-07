use lugli_common::Value;
use std::{cell::RefCell, rc::Rc};

#[test]
fn test_value_equality_refactored() {
    // Test that PartialEq and equals() produce identical results
    let v1 = Value::Number(42.0);
    let v2 = Value::Number(42.0);
    let v3 = Value::Number(43.0);

    assert_eq!(v1 == v2, v1.equals(&v2));
    assert_eq!(v1 == v3, v1.equals(&v3));

    use lugli_common::StringPool;
    let mut pool = StringPool::new();
    let s1 = Value::String(pool.intern("hello"));
    let s2 = Value::String(pool.intern("hello"));
    let s3 = Value::String(pool.intern("world"));

    assert_eq!(s1 == s2, s1.equals(&s2));
    assert_eq!(s1 == s3, s1.equals(&s3));
}

#[test]
fn test_dict_equality_deep_equals() {
    use hashbrown::HashMap;
    use lugli_common::StringPool;

    let mut pool = StringPool::new();
    let key1 = pool.intern("x");
    let key2 = pool.intern("y");

    let dict1 = {
        let mut map = HashMap::new();
        map.insert(key1, Value::Number(1.0));
        map.insert(key2, Value::Number(2.0));
        Value::Dict(Rc::new(RefCell::new(map)))
    };

    let dict2 = {
        let mut map = HashMap::new();
        map.insert(key1, Value::Number(1.0));
        map.insert(key2, Value::Number(2.0));
        Value::Dict(Rc::new(RefCell::new(map)))
    };

    // Both == and equals() should use deep equality via recursive equals()
    assert!(dict1.equals(&dict2));
    assert!(dict1 == dict2);
}

#[test]
fn test_list_equality_ptr_based() {
    let list1 = Value::List(Rc::new(RefCell::new(vec![Value::Number(1.0)])));
    let list2 = Value::List(Rc::new(RefCell::new(vec![Value::Number(1.0)])));
    let list3 = list1.clone();

    // Different Rc pointers = not equal
    assert!(!list1.equals(&list2));
    assert!(!(list1 == list2));

    // Same Rc pointer = equal
    assert!(list1.equals(&list3));
    assert!(list1 == list3);
}

#[test]
fn test_closure_equality_with_upvalues() {
    let upval1 = Rc::new(RefCell::new(Value::Number(10.0)));
    let upval2 = Rc::new(RefCell::new(Value::Number(10.0)));

    let closure1 = Value::Closure {
        name: Rc::from("test"),
        params: Rc::new(vec!["x".to_string()]),
        body_start: 100,
        bytecode_id: 0,
        upvalues: vec![upval1.clone()],
        required_count: 1,
        defaults: Rc::new(vec![]),
    };

    let closure2 = Value::Closure {
        name: Rc::from("test"),
        params: Rc::new(vec!["x".to_string()]),
        body_start: 100,
        bytecode_id: 0,
        upvalues: vec![upval1.clone()],
        required_count: 1,
        defaults: Rc::new(vec![]),
    };

    let closure3 = Value::Closure {
        name: Rc::from("test"),
        params: Rc::new(vec!["x".to_string()]),
        body_start: 100,
        bytecode_id: 0,
        upvalues: vec![upval2],
        required_count: 1,
        defaults: Rc::new(vec![]),
    };

    // Same upvalue Rc = equal
    assert!(closure1.equals(&closure2));
    assert!(closure1 == closure2);

    // Different upvalue Rc but same value = equal
    assert!(closure1.equals(&closure3));
    assert!(closure1 == closure3);
}
