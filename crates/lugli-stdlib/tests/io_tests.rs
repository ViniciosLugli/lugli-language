use lugli_common::Value;
use lugli_stdlib::get_global_functions;

mod io_function_tests {
    use super::*;

    #[test]
    fn test_print_function_exists() {
        let functions = get_global_functions();
        let print_fn = functions.iter()
            .find(|(name, _)| *name == "print")
            .map(|(_, func)| func)
            .expect("print function should exist");

        // Test that print function executes without error
        let result = print_fn(&[Value::String("hello".to_string())]);
        assert!(result.is_ok());
        assert!(result.unwrap().equals(&Value::Null));

        // Test print with multiple arguments
        let result = print_fn(&[
            Value::String("hello".to_string()),
            Value::Number(42.0),
            Value::Bool(true)
        ]);
        assert!(result.is_ok());
        assert!(result.unwrap().equals(&Value::Null));

        // Test print with no arguments
        let result = print_fn(&[]);
        assert!(result.is_ok());
        assert!(result.unwrap().equals(&Value::Null));
    }

    #[test]
    fn test_println_function_exists() {
        let functions = get_global_functions();
        let println_fn = functions.iter()
            .find(|(name, _)| *name == "println")
            .map(|(_, func)| func)
            .expect("println function should exist");

        // Test that println function executes without error
        let result = println_fn(&[Value::String("hello world".to_string())]);
        assert!(result.is_ok());
        assert!(result.unwrap().equals(&Value::Null));

        // Test println with multiple arguments
        let result = println_fn(&[
            Value::String("test".to_string()),
            Value::Number(123.0)
        ]);
        assert!(result.is_ok());
        assert!(result.unwrap().equals(&Value::Null));

        // Test println with no arguments (should print empty line)
        let result = println_fn(&[]);
        assert!(result.is_ok());
        assert!(result.unwrap().equals(&Value::Null));
    }

    #[test]
    fn test_io_functions_return_null() {
        let functions = get_global_functions();

        let print_fn = functions.iter()
            .find(|(name, _)| *name == "print")
            .map(|(_, func)| func)
            .unwrap();

        let println_fn = functions.iter()
            .find(|(name, _)| *name == "println")
            .map(|(_, func)| func)
            .unwrap();

        // Both functions should return Null
        let print_result = print_fn(&[Value::String("test".to_string())]).unwrap();
        let println_result = println_fn(&[Value::String("test".to_string())]).unwrap();

        assert!(print_result.equals(&Value::Null));
        assert!(println_result.equals(&Value::Null));
    }
}