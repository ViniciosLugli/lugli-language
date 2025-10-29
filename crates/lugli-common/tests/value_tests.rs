use hashbrown::HashMap;
use lugli_common::{StringPool, Value};
use std::{cell::RefCell, rc::Rc};

mod basic_tests {
    use super::*;

    #[test]
    fn test_value_creation() {
        let mut pool = StringPool::new();
        let number = Value::Number(42.0);
        let string = Value::String(pool.intern("hello"));
        let boolean = Value::Bool(true);
        let null = Value::Null;

        assert_eq!(number.type_name(), "number");
        assert_eq!(string.type_name(), "string");
        assert_eq!(boolean.type_name(), "bool");
        assert_eq!(null.type_name(), "null");
    }

    #[test]
    fn test_value_equality() {
        let mut pool = StringPool::new();
        let num1 = Value::Number(42.0);
        let num2 = Value::Number(42.0);
        let num3 = Value::Number(43.0);

        assert!(num1.equals(&num2));
        assert!(!num1.equals(&num3));

        let str1 = Value::String(pool.intern("test"));
        let str2 = Value::String(pool.intern("test"));
        let str3 = Value::String(pool.intern("other"));

        assert!(str1.equals(&str2));
        assert!(!str1.equals(&str3));

        let bool1 = Value::Bool(true);
        let bool2 = Value::Bool(true);
        let bool3 = Value::Bool(false);

        assert!(bool1.equals(&bool2));
        assert!(!bool1.equals(&bool3));

        let null1 = Value::Null;
        let null2 = Value::Null;

        assert!(null1.equals(&null2));
    }

    #[test]
    fn test_value_truthiness() {
        let mut pool = StringPool::new();
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(!Value::Null.is_truthy());
        assert!(Value::Number(1.0).is_truthy());
        assert!(!Value::Number(0.0).is_truthy());
        assert!(Value::String(pool.intern("hello")).is_truthy_with_pool(&pool));
        assert!(!Value::String(pool.intern("")).is_truthy_with_pool(&pool));
    }
}

mod collection_tests {
    use super::*;

    #[test]
    fn test_list_creation() {
        let elements = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
        let list = Value::List(Rc::new(RefCell::new(elements)));

        assert_eq!(list.type_name(), "list");

        if let Value::List(list_ref) = list {
            let borrowed = list_ref.borrow();
            assert_eq!(borrowed.len(), 3);
            assert!(borrowed[0].equals(&Value::Number(1.0)));
        } else {
            panic!("Expected List value");
        }
    }

    #[test]
    fn test_dict_creation() {
        let mut pool = StringPool::new();
        let mut dict = HashMap::new();
        let name_key = pool.intern("name");
        let age_key = pool.intern("age");
        dict.insert(name_key, Value::String(pool.intern("Alice")));
        dict.insert(age_key, Value::Number(30.0));

        let dict_value = Value::Dict(Rc::new(RefCell::new(dict)));
        assert_eq!(dict_value.type_name(), "dict");

        if let Value::Dict(dict_ref) = dict_value {
            let borrowed = dict_ref.borrow();
            assert_eq!(borrowed.len(), 2);
            assert!(borrowed.get(&name_key).unwrap().equals(&Value::String(pool.intern("Alice"))));
            assert!(borrowed.get(&age_key).unwrap().equals(&Value::Number(30.0)));
        } else {
            panic!("Expected Dict value");
        }
    }

    #[test]
    fn test_dict_equality() {
        let mut pool = StringPool::new();
        let key = pool.intern("key");

        let mut dict1_map = HashMap::new();
        dict1_map.insert(key, Value::Number(42.0));
        let dict1 = Value::Dict(Rc::new(RefCell::new(dict1_map)));

        let mut dict2_map = HashMap::new();
        dict2_map.insert(key, Value::Number(42.0));
        let dict2 = Value::Dict(Rc::new(RefCell::new(dict2_map)));

        let mut dict3_map = HashMap::new();
        dict3_map.insert(key, Value::Number(43.0));
        let dict3 = Value::Dict(Rc::new(RefCell::new(dict3_map)));

        assert!(!dict1.equals(&dict2));
        assert!(!dict1.equals(&dict3));
    }
}

mod function_tests {
    use super::*;
    use lugli_common::LugliError;

    fn test_native_function(_args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> { Ok(Value::String(pool.intern("test result"))) }

    #[test]
    fn test_native_function_creation() {
        let native_fn = Value::NativeFunction {
            name: "test_fn".to_string(),
            callback: test_native_function,
            arity: 0,
        };

        assert_eq!(native_fn.type_name(), "function");

        if let Value::NativeFunction {
            name,
            arity,
            ..
        } = native_fn
        {
            assert_eq!(name, "test_fn");
            assert_eq!(arity, 0);
        } else {
            panic!("Expected NativeFunction value");
        }
    }

    #[test]
    fn test_function_creation() {
        let function = Value::Function {
            name: "my_function".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            body_start: 0,
            bytecode_id: 0,
        };

        assert_eq!(function.type_name(), "function");

        if let Value::Function {
            name,
            params,
            body_start,
            ..
        } = function
        {
            assert_eq!(name, "my_function");
            assert_eq!(params.len(), 2);
            assert_eq!(body_start, 0);
        } else {
            panic!("Expected Function value");
        }
    }
}
#[test]
fn test_division_by_zero() {
    let a = Value::Number(10.0);
    let b = Value::Number(0.0);

    // Regular division
    let result = a.divide(&b);
    assert!(result.is_err(), "Division by zero should error");
    assert!(result.unwrap_err().to_string().contains("Division by zero"));

    // Integer division
    let result = a.integer_divide(&b);
    assert!(result.is_err(), "Integer division by zero should error");
    assert!(result.unwrap_err().to_string().contains("Division by zero"));

    // Modulo
    let result = a.modulo(&b);
    assert!(result.is_err(), "Modulo by zero should error");
    assert!(result.unwrap_err().to_string().contains("zero"));
}

#[test]
fn test_division_by_near_zero() {
    let a = Value::Number(1.0);
    let b = Value::Number(1e-308); // Near zero but not zero

    let result = a.divide(&b);
    assert!(result.is_ok(), "Division by near-zero should succeed");

    // Result should be very large
    if let Ok(Value::Number(val)) = result {
        assert!(val > 1e307, "Result should be very large");
    }
}

#[test]
fn test_division_special_cases() {
    // Positive / Positive
    let result = Value::Number(10.0).divide(&Value::Number(2.0)).unwrap();
    assert_eq!(result, Value::Number(5.0));

    // Negative / Positive
    let result = Value::Number(-10.0).divide(&Value::Number(2.0)).unwrap();
    assert_eq!(result, Value::Number(-5.0));

    // Positive / Negative
    let result = Value::Number(10.0).divide(&Value::Number(-2.0)).unwrap();
    assert_eq!(result, Value::Number(-5.0));

    // Negative / Negative
    let result = Value::Number(-10.0).divide(&Value::Number(-2.0)).unwrap();
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_modulo_special_cases() {
    // Positive % Positive
    let result = Value::Number(10.0).modulo(&Value::Number(3.0)).unwrap();
    assert_eq!(result, Value::Number(1.0));

    // Negative % Positive (Rust semantics: result has sign of dividend)
    let result = Value::Number(-10.0).modulo(&Value::Number(3.0)).unwrap();
    assert_eq!(result, Value::Number(-1.0));

    // Modulo by zero
    let result = Value::Number(10.0).modulo(&Value::Number(0.0));
    assert!(result.is_err());
}
