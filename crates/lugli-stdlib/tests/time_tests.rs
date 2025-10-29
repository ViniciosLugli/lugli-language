use lugli_common::Value;
use lugli_stdlib::get_global_functions;
use std::time::Instant;

mod time_function_tests {
    use super::*;

    #[test]
    fn test_now_function() {
        let functions = get_global_functions();
        let now_fn = functions.iter().find(|(name, _)| *name == "now").map(|(_, func)| func).expect("now function should exist");

        // Test that now() returns a number (timestamp)
        let result = now_fn(&[]);
        assert!(result.is_ok());

        let value = result.unwrap();
        match value {
            Value::Number(timestamp) => {
                // Timestamp should be positive and reasonable (after year 2000)
                assert!(timestamp > 946684800.0); // Jan 1, 2000
                assert!(timestamp < 4102444800.0); // Jan 1, 2100
            }
            _ => panic!("now() should return a number"),
        }

        // Test that calling now() twice gives different timestamps
        let result1 = now_fn(&[]).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let result2 = now_fn(&[]).unwrap();

        if let (Value::Number(t1), Value::Number(t2)) = (result1, result2) {
            assert!(t2 >= t1); // Second timestamp should be >= first
        }
    }

    #[test]
    fn test_sleep_function() {
        let functions = get_global_functions();
        let sleep_fn = functions.iter().find(|(name, _)| *name == "sleep").map(|(_, func)| func).expect("sleep function should exist");

        // Test that sleep with small duration works
        let start = Instant::now();
        let result = sleep_fn(&[Value::Number(10.0)]); // 10ms
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(result.unwrap().equals(&Value::Null));
        assert!(elapsed >= std::time::Duration::from_millis(10));
        assert!(elapsed < std::time::Duration::from_millis(50)); // Should be reasonably fast

        // Test error cases
        let error_result = sleep_fn(&[]);
        assert!(error_result.is_err());

        let error_result = sleep_fn(&[Value::Number(1.0), Value::Number(2.0)]);
        assert!(error_result.is_err());

        let error_result = sleep_fn(&[Value::String("not a number".to_string())]);
        assert!(error_result.is_err());
    }

    #[test]
    fn test_time_function_arguments() {
        let functions = get_global_functions();

        let now_fn = functions.iter().find(|(name, _)| *name == "now").map(|(_, func)| func).unwrap();

        let sleep_fn = functions.iter().find(|(name, _)| *name == "sleep").map(|(_, func)| func).unwrap();

        // now() should work with any number of arguments (ignores them)
        let result = now_fn(&[Value::Number(42.0)]);
        assert!(result.is_ok());

        // sleep() requires exactly 1 argument
        let result = sleep_fn(&[Value::Number(0.0)]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sleep_with_zero() {
        let functions = get_global_functions();
        let sleep_fn = functions.iter().find(|(name, _)| *name == "sleep").map(|(_, func)| func).unwrap();

        // Test sleep with 0 duration
        let result = sleep_fn(&[Value::Number(0.0)]);
        assert!(result.is_ok());
        assert!(result.unwrap().equals(&Value::Null));
    }
}
