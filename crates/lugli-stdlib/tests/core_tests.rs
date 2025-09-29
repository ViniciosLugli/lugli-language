use lugli_common::Value;
use lugli_stdlib::get_global_functions;
use hashbrown::HashMap;
use std::{cell::RefCell, rc::Rc};

mod core_function_tests {
    use super::*;

    #[test]
    fn test_type_function() {
        let functions = get_global_functions();
        let type_fn = functions.iter()
            .find(|(name, _)| *name == "type")
            .map(|(_, func)| func)
            .expect("type function should exist");

        // Test with different value types
        let number_result = type_fn(&[Value::Number(42.0)]).unwrap();
        assert!(number_result.equals(&Value::String("number".to_string())));

        let string_result = type_fn(&[Value::String("hello".to_string())]).unwrap();
        assert!(string_result.equals(&Value::String("string".to_string())));

        let bool_result = type_fn(&[Value::Bool(true)]).unwrap();
        assert!(bool_result.equals(&Value::String("bool".to_string())));

        let null_result = type_fn(&[Value::Null]).unwrap();
        assert!(null_result.equals(&Value::String("null".to_string())));

        // Test error case - wrong number of arguments
        let error_result = type_fn(&[]);
        assert!(error_result.is_err());

        let error_result = type_fn(&[Value::Number(1.0), Value::Number(2.0)]);
        assert!(error_result.is_err());
    }

    #[test]
    fn test_len_function() {
        let functions = get_global_functions();
        let len_fn = functions.iter()
            .find(|(name, _)| *name == "len")
            .map(|(_, func)| func)
            .expect("len function should exist");

        // Test string length
        let string_result = len_fn(&[Value::String("hello".to_string())]).unwrap();
        assert!(string_result.equals(&Value::Number(5.0)));

        let empty_string_result = len_fn(&[Value::String("".to_string())]).unwrap();
        assert!(empty_string_result.equals(&Value::Number(0.0)));

        // Test list length
        let list = Value::List(Rc::new(RefCell::new(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0)
        ])));
        let list_result = len_fn(&[list]).unwrap();
        assert!(list_result.equals(&Value::Number(3.0)));

        // Test dict length
        let mut dict = HashMap::new();
        dict.insert("key1".to_string(), Value::Number(1.0));
        dict.insert("key2".to_string(), Value::Number(2.0));
        let dict_value = Value::Dict(dict);
        let dict_result = len_fn(&[dict_value]).unwrap();
        assert!(dict_result.equals(&Value::Number(2.0)));

        // Test error cases
        let error_result = len_fn(&[Value::Number(42.0)]);
        assert!(error_result.is_err());

        let error_result = len_fn(&[]);
        assert!(error_result.is_err());
    }

    #[test]
    fn test_function_registry() {
        let functions = get_global_functions();

        // Check that core functions are registered
        let function_names: Vec<&str> = functions.iter().map(|(name, _)| *name).collect();

        assert!(function_names.contains(&"type"));
        assert!(function_names.contains(&"len"));
        assert!(function_names.contains(&"print"));
        assert!(function_names.contains(&"println"));
        assert!(function_names.contains(&"now"));
        assert!(function_names.contains(&"sleep"));
    }
}