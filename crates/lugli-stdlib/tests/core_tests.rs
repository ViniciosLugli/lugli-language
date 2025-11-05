use hashbrown::HashMap;
use lugli_common::{StringPool, Value};
use lugli_stdlib::get_global_functions;
use std::{cell::RefCell, rc::Rc};

mod core_function_tests {
    use super::*;

    #[test]
    fn test_type_function() {
        let mut pool = StringPool::new();
        let functions = get_global_functions();
        let type_fn = functions.iter().find(|(name, _)| *name == "type").map(|(_, func)| func).expect("type function should exist");

        let number_result = type_fn(&[Value::Number(42.0)], &mut pool).unwrap();
        if let Value::String(id) = number_result {
            assert_eq!(pool.resolve(id), "number");
        }

        let string_result = type_fn(&[Value::String(pool.intern("hello"))], &mut pool).unwrap();
        if let Value::String(id) = string_result {
            assert_eq!(pool.resolve(id), "string");
        }

        let bool_result = type_fn(&[Value::Bool(true)], &mut pool).unwrap();
        if let Value::String(id) = bool_result {
            assert_eq!(pool.resolve(id), "bool");
        }

        let null_result = type_fn(&[Value::Null], &mut pool).unwrap();
        if let Value::String(id) = null_result {
            assert_eq!(pool.resolve(id), "null");
        }

        let error_result = type_fn(&[], &mut pool);
        assert!(error_result.is_err());

        let error_result = type_fn(&[Value::Number(1.0), Value::Number(2.0)], &mut pool);
        assert!(error_result.is_err());
    }

    #[test]
    fn test_len_function() {
        let mut pool = StringPool::new();
        let functions = get_global_functions();
        let len_fn = functions.iter().find(|(name, _)| *name == "len").map(|(_, func)| func).expect("len function should exist");

        let string_result = len_fn(&[Value::String(pool.intern("hello"))], &mut pool).unwrap();
        assert!(string_result.equals(&Value::Number(5.0)));

        let empty_string_result = len_fn(&[Value::String(pool.intern(""))], &mut pool).unwrap();
        assert!(empty_string_result.equals(&Value::Number(0.0)));

        let list = Value::List(Rc::new(RefCell::new(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)])));
        let list_result = len_fn(&[list], &mut pool).unwrap();
        assert!(list_result.equals(&Value::Number(3.0)));

        let mut dict = HashMap::new();
        dict.insert(pool.intern("key1"), Value::Number(1.0));
        dict.insert(pool.intern("key2"), Value::Number(2.0));
        let dict_value = Value::Dict(Rc::new(RefCell::new(dict)));
        let dict_result = len_fn(&[dict_value], &mut pool).unwrap();
        assert!(dict_result.equals(&Value::Number(2.0)));

        let error_result = len_fn(&[Value::Number(42.0)], &mut pool);
        assert!(error_result.is_err());

        let error_result = len_fn(&[], &mut pool);
        assert!(error_result.is_err());
    }

    #[test]
    fn test_function_registry() {
        let functions = get_global_functions();

        let function_names: Vec<&str> = functions.iter().map(|(name, _)| *name).collect();

        assert!(function_names.contains(&"type"));
        assert!(function_names.contains(&"len"));
        assert!(function_names.contains(&"print"));
        assert!(function_names.contains(&"now"));
        assert!(function_names.contains(&"sleep"));
    }
}
