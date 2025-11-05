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

    #[test]
    fn test_ceil_function() {
        let mut pool = StringPool::new();
        let functions = get_global_functions();
        let ceil_fn = functions.iter().find(|(name, _)| *name == "ceil").map(|(_, func)| func).expect("ceil function should exist");

        let result = ceil_fn(&[Value::Number(3.2)], &mut pool).unwrap();
        assert!(result.equals(&Value::Number(4.0)));

        let result = ceil_fn(&[Value::Number(-3.8)], &mut pool).unwrap();
        assert!(result.equals(&Value::Number(-3.0)));

        let result = ceil_fn(&[Value::Number(5.0)], &mut pool).unwrap();
        assert!(result.equals(&Value::Number(5.0)));

        let error_result = ceil_fn(&[Value::String(pool.intern("not a number"))], &mut pool);
        assert!(error_result.is_err());
    }

    #[test]
    fn test_floor_function() {
        let mut pool = StringPool::new();
        let functions = get_global_functions();
        let floor_fn = functions.iter().find(|(name, _)| *name == "floor").map(|(_, func)| func).expect("floor function should exist");

        let result = floor_fn(&[Value::Number(3.8)], &mut pool).unwrap();
        assert!(result.equals(&Value::Number(3.0)));

        let result = floor_fn(&[Value::Number(-3.2)], &mut pool).unwrap();
        assert!(result.equals(&Value::Number(-4.0)));

        let result = floor_fn(&[Value::Number(5.0)], &mut pool).unwrap();
        assert!(result.equals(&Value::Number(5.0)));
    }

    #[test]
    fn test_sqrt_function() {
        let mut pool = StringPool::new();
        let functions = get_global_functions();
        let sqrt_fn = functions.iter().find(|(name, _)| *name == "sqrt").map(|(_, func)| func).expect("sqrt function should exist");

        let result = sqrt_fn(&[Value::Number(16.0)], &mut pool).unwrap();
        assert!(result.equals(&Value::Number(4.0)));

        let result = sqrt_fn(&[Value::Number(0.0)], &mut pool).unwrap();
        assert!(result.equals(&Value::Number(0.0)));

        let error_result = sqrt_fn(&[Value::Number(-1.0)], &mut pool);
        assert!(error_result.is_err());
    }

    #[test]
    fn test_sin_function() {
        let mut pool = StringPool::new();
        let functions = get_global_functions();
        let sin_fn = functions.iter().find(|(name, _)| *name == "sin").map(|(_, func)| func).expect("sin function should exist");

        let result = sin_fn(&[Value::Number(0.0)], &mut pool).unwrap();
        if let Value::Number(n) = result {
            assert!((n - 0.0).abs() < 1e-10);
        } else {
            panic!("Expected number");
        }

        let result = sin_fn(&[Value::Number(std::f64::consts::PI / 2.0)], &mut pool).unwrap();
        if let Value::Number(n) = result {
            assert!((n - 1.0).abs() < 1e-10);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_cos_function() {
        let mut pool = StringPool::new();
        let functions = get_global_functions();
        let cos_fn = functions.iter().find(|(name, _)| *name == "cos").map(|(_, func)| func).expect("cos function should exist");

        let result = cos_fn(&[Value::Number(0.0)], &mut pool).unwrap();
        if let Value::Number(n) = result {
            assert!((n - 1.0).abs() < 1e-10);
        } else {
            panic!("Expected number");
        }

        let result = cos_fn(&[Value::Number(std::f64::consts::PI)], &mut pool).unwrap();
        if let Value::Number(n) = result {
            assert!((n + 1.0).abs() < 1e-10);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_log_function() {
        let mut pool = StringPool::new();
        let functions = get_global_functions();
        let log_fn = functions.iter().find(|(name, _)| *name == "log").map(|(_, func)| func).expect("log function should exist");

        let result = log_fn(&[Value::Number(std::f64::consts::E)], &mut pool).unwrap();
        if let Value::Number(n) = result {
            assert!((n - 1.0).abs() < 1e-10);
        } else {
            panic!("Expected number");
        }

        let error_result = log_fn(&[Value::Number(0.0)], &mut pool);
        assert!(error_result.is_err());

        let error_result = log_fn(&[Value::Number(-1.0)], &mut pool);
        assert!(error_result.is_err());
    }

    #[test]
    fn test_exp_function() {
        let mut pool = StringPool::new();
        let functions = get_global_functions();
        let exp_fn = functions.iter().find(|(name, _)| *name == "exp").map(|(_, func)| func).expect("exp function should exist");

        let result = exp_fn(&[Value::Number(0.0)], &mut pool).unwrap();
        if let Value::Number(n) = result {
            assert!((n - 1.0).abs() < 1e-10);
        } else {
            panic!("Expected number");
        }

        let result = exp_fn(&[Value::Number(1.0)], &mut pool).unwrap();
        if let Value::Number(n) = result {
            assert!((n - std::f64::consts::E).abs() < 1e-10);
        } else {
            panic!("Expected number");
        }
    }
}
