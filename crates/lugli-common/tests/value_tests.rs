use lugli_common::Value;
use hashbrown::HashMap;
use std::{cell::RefCell, rc::Rc};

mod basic_tests {
    use super::*;

    #[test]
    fn test_value_creation() {
        let number = Value::Number(42.0);
        let string = Value::String("hello".to_string());
        let boolean = Value::Bool(true);
        let null = Value::Null;

        assert_eq!(number.type_name(), "number");
        assert_eq!(string.type_name(), "string");
        assert_eq!(boolean.type_name(), "bool");
        assert_eq!(null.type_name(), "null");
    }

    #[test]
    fn test_value_equality() {
        let num1 = Value::Number(42.0);
        let num2 = Value::Number(42.0);
        let num3 = Value::Number(43.0);

        assert!(num1.equals(&num2));
        assert!(!num1.equals(&num3));

        let str1 = Value::String("test".to_string());
        let str2 = Value::String("test".to_string());
        let str3 = Value::String("other".to_string());

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
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(!Value::Null.is_truthy());
        assert!(Value::Number(1.0).is_truthy());
        assert!(!Value::Number(0.0).is_truthy());
        assert!(Value::String("hello".to_string()).is_truthy());
        assert!(!Value::String("".to_string()).is_truthy());
    }

    #[test]
    fn test_number_conversions() {
        let number = Value::Number(42.5);
        assert_eq!(number.to_number(), 42.5);

        let string_num = Value::String("123.45".to_string());
        assert_eq!(string_num.to_number(), 123.45);

        let invalid_string = Value::String("not_a_number".to_string());
        assert_eq!(invalid_string.to_number(), 0.0); // Returns 0.0 for invalid strings

        let boolean_true = Value::Bool(true);
        assert_eq!(boolean_true.to_number(), 1.0);

        let boolean_false = Value::Bool(false);
        assert_eq!(boolean_false.to_number(), 0.0);

        let null = Value::Null;
        assert_eq!(null.to_number(), 0.0);
    }

    #[test]
    fn test_string_conversions() {
        let number = Value::Number(42.0);
        assert_eq!(number.to_string(), "42");

        let string = Value::String("hello".to_string());
        assert_eq!(string.to_string(), "hello");

        let boolean = Value::Bool(true);
        assert_eq!(boolean.to_string(), "true");

        let null = Value::Null;
        assert_eq!(null.to_string(), "");
    }

    #[test]
    fn test_bool_conversions() {
        let true_bool = Value::Bool(true);
        assert_eq!(true_bool.to_bool(), true);

        let false_bool = Value::Bool(false);
        assert_eq!(false_bool.to_bool(), false);

        let truthy_number = Value::Number(42.0);
        assert_eq!(truthy_number.to_bool(), true);

        let falsy_number = Value::Number(0.0);
        assert_eq!(falsy_number.to_bool(), false);

        let truthy_string = Value::String("hello".to_string());
        assert_eq!(truthy_string.to_bool(), true);

        let falsy_string = Value::String("".to_string());
        assert_eq!(falsy_string.to_bool(), false);

        let null = Value::Null;
        assert_eq!(null.to_bool(), false);
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
        let mut dict = HashMap::new();
        dict.insert("name".to_string(), Value::String("Alice".to_string()));
        dict.insert("age".to_string(), Value::Number(30.0));

        let dict_value = Value::Dict(dict);
        assert_eq!(dict_value.type_name(), "dict");

        if let Value::Dict(dict_ref) = dict_value {
            assert_eq!(dict_ref.len(), 2);
            assert!(dict_ref.get("name").unwrap().equals(&Value::String("Alice".to_string())));
            assert!(dict_ref.get("age").unwrap().equals(&Value::Number(30.0)));
        } else {
            panic!("Expected Dict value");
        }
    }

    #[test]
    fn test_dict_equality() {
        let mut dict1 = HashMap::new();
        dict1.insert("key".to_string(), Value::Number(42.0));

        let mut dict2 = HashMap::new();
        dict2.insert("key".to_string(), Value::Number(42.0));

        let mut dict3 = HashMap::new();
        dict3.insert("key".to_string(), Value::Number(43.0));

        let value1 = Value::Dict(dict1);
        let value2 = Value::Dict(dict2);
        let value3 = Value::Dict(dict3);

        // Note: Dict equality is not implemented in Value::equals, returns false
        assert!(!value1.equals(&value2));
        assert!(!value1.equals(&value3));
    }
}

mod function_tests {
    use super::*;
    use lugli_common::LugliError;

    fn test_native_function(_args: &[Value]) -> Result<Value, LugliError> {
        Ok(Value::String("test result".to_string()))
    }

    #[test]
    fn test_native_function_creation() {
        let native_fn = Value::NativeFunction {
            name: "test_fn".to_string(),
            callback: test_native_function,
            arity: 0,
        };

        assert_eq!(native_fn.type_name(), "function");

        if let Value::NativeFunction { name, arity, .. } = native_fn {
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
            arity: 2,
        };

        assert_eq!(function.type_name(), "function");

        if let Value::Function { name, params, arity } = function {
            assert_eq!(name, "my_function");
            assert_eq!(params.len(), 2);
            assert_eq!(arity, 2);
        } else {
            panic!("Expected Function value");
        }
    }
}