use lugli_common::{StringPool, Value};
use lugli_stdlib::get_global_functions;

mod io_function_tests {
    use super::*;

    #[test]
    fn test_print_function_exists() {
        let mut pool = StringPool::new();
        let functions = get_global_functions();
        let print_fn = functions.iter().find(|(name, _)| *name == "print").map(|(_, func)| func).expect("print function should exist");

        let result = print_fn(&[Value::String(pool.intern("hello"))], &mut pool);
        assert!(result.is_ok());
        assert!(result.unwrap().equals(&Value::Null));

        let result = print_fn(&[Value::String(pool.intern("hello")), Value::Number(42.0), Value::Bool(true)], &mut pool);
        assert!(result.is_ok());
        assert!(result.unwrap().equals(&Value::Null));

        let result = print_fn(&[], &mut pool);
        assert!(result.is_ok());
        assert!(result.unwrap().equals(&Value::Null));
    }

    #[test]
    fn test_print_returns_null() {
        let mut pool = StringPool::new();
        let functions = get_global_functions();

        let print_fn = functions.iter().find(|(name, _)| *name == "print").map(|(_, func)| func).unwrap();

        let print_result = print_fn(&[Value::String(pool.intern("test"))], &mut pool).unwrap();

        assert!(print_result.equals(&Value::Null));
    }
}
