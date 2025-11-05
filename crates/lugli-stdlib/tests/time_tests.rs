use lugli_common::{StringPool, Value};
use lugli_stdlib::get_global_functions;
use std::time::Instant;

mod time_function_tests {
    use super::*;

    #[test]
    fn test_now_function() {
        let mut pool = StringPool::new();
        let functions = get_global_functions();
        let now_fn = functions.iter().find(|(name, _)| *name == "now").map(|(_, func)| func).expect("now function should exist");

        let result = now_fn(&[], &mut pool);
        assert!(result.is_ok());

        let value = result.unwrap();
        match value {
            Value::Number(timestamp) => {
                assert!(timestamp > 946684800.0);
                assert!(timestamp < 4102444800.0);
            }
            _ => panic!("now() should return a number"),
        }

        let result1 = now_fn(&[], &mut pool).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let result2 = now_fn(&[], &mut pool).unwrap();

        if let (Value::Number(t1), Value::Number(t2)) = (result1, result2) {
            assert!(t2 >= t1);
        }
    }

    #[test]
    fn test_sleep_function() {
        let mut pool = StringPool::new();
        let functions = get_global_functions();
        let sleep_fn = functions.iter().find(|(name, _)| *name == "sleep").map(|(_, func)| func).expect("sleep function should exist");

        let start = Instant::now();
        let result = sleep_fn(&[Value::Number(10.0)], &mut pool);
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(result.unwrap().equals(&Value::Null));
        // Check minimum sleep time
        assert!(elapsed >= std::time::Duration::from_millis(10));
        // More lenient upper bound for CI environments (especially macOS)
        // Sleep can take longer due to scheduler overhead, context switches, etc.
        assert!(elapsed < std::time::Duration::from_millis(200), "Sleep took {:?}, expected < 200ms", elapsed);

        let error_result = sleep_fn(&[], &mut pool);
        assert!(error_result.is_err());

        let error_result = sleep_fn(&[Value::Number(1.0), Value::Number(2.0)], &mut pool);
        assert!(error_result.is_err());

        let error_result = sleep_fn(&[Value::String(pool.intern("not a number"))], &mut pool);
        assert!(error_result.is_err());
    }

    #[test]
    fn test_time_function_arguments() {
        let mut pool = StringPool::new();
        let functions = get_global_functions();

        let now_fn = functions.iter().find(|(name, _)| *name == "now").map(|(_, func)| func).unwrap();

        let sleep_fn = functions.iter().find(|(name, _)| *name == "sleep").map(|(_, func)| func).unwrap();

        let result = now_fn(&[Value::Number(42.0)], &mut pool);
        assert!(result.is_ok());

        let result = sleep_fn(&[Value::Number(0.0)], &mut pool);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sleep_with_zero() {
        let mut pool = StringPool::new();
        let functions = get_global_functions();
        let sleep_fn = functions.iter().find(|(name, _)| *name == "sleep").map(|(_, func)| func).unwrap();

        let result = sleep_fn(&[Value::Number(0.0)], &mut pool);
        assert!(result.is_ok());
        assert!(result.unwrap().equals(&Value::Null));
    }
}
