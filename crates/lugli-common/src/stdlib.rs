//! Standard library trait abstraction.
//!
//! Provides traits for pluggable standard library implementations.

use crate::{LugliError, NativeFunction, StringPool, Value};

/// Trait for standard library implementations.
///
/// This allows swapping out the standard library with custom implementations
/// for testing, embedding, or providing alternative functionality.
pub trait StandardLibrary: Send + Sync {
    /// Get all global functions (print, len, type, etc.)
    ///
    /// Returns a vector of (name, function) pairs that will be registered
    /// as global functions in the VM.
    fn get_global_functions(&self) -> Vec<(&'static str, NativeFunction)>;

    /// Get method registry for type methods
    ///
    /// Returns a boxed trait object that implements MethodRegistry.
    /// This registry handles method dispatch for built-in types.
    fn get_method_registry(&self) -> Box<dyn MethodRegistry>;

    /// Get library name and version (for introspection)
    fn info(&self) -> (&'static str, &'static str);
}

/// Trait for method registry implementations.
///
/// Handles method dispatch for built-in types (String, List, Dict, etc.).
pub trait MethodRegistry: Send + Sync {
    /// Call a method on a value
    ///
    /// # Arguments
    /// * `type_name` - The type name (e.g., "string", "list")
    /// * `method_name` - The method name (e.g., "upper", "push")
    /// * `args` - Arguments including self as first argument
    /// * `pool` - String pool for string operations
    fn call(&self, type_name: &str, method_name: &str, args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError>;

    /// Call a method by precomputed hash (for performance)
    ///
    /// This is used by the VM's method cache to avoid repeated hash computations.
    fn call_by_hash(&self, hash: u32, args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError>;

    /// Check if a method exists for a type
    fn has_method(&self, type_name: &str, method_name: &str) -> bool;

    /// Get method ID for caching (optional optimization)
    fn get_method_id(&self, type_name: &str, method_name: &str) -> Option<u32>;
}

/// Factory trait for creating standard library instances
pub trait StandardLibraryFactory {
    /// Create a new instance of the standard library
    fn create() -> Box<dyn StandardLibrary>
    where Self: Sized;
}

/// Minimal test implementation for testing
#[cfg(test)]
pub struct TestStdlib;

#[cfg(test)]
impl StandardLibrary for TestStdlib {
    fn get_global_functions(&self) -> Vec<(&'static str, NativeFunction)> {
        vec![
            ("test_fn", |_args, _pool| Ok(Value::Number(42.0))),
            ("echo", |args, _pool| if args.is_empty() { Ok(Value::Null) } else { Ok(args[0].clone()) }),
        ]
    }

    fn get_method_registry(&self) -> Box<dyn MethodRegistry> { Box::new(EmptyMethodRegistry) }

    fn info(&self) -> (&'static str, &'static str) { ("test-stdlib", "0.0.1") }
}

#[cfg(test)]
struct EmptyMethodRegistry;

#[cfg(test)]
impl MethodRegistry for EmptyMethodRegistry {
    fn call(&self, _type_name: &str, _method_name: &str, _args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
        Err(LugliError::runtime("No methods available in test registry"))
    }

    fn call_by_hash(&self, _hash: u32, _args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
        Err(LugliError::runtime("No methods available in test registry"))
    }

    fn has_method(&self, _type_name: &str, _method_name: &str) -> bool { false }

    fn get_method_id(&self, _type_name: &str, _method_name: &str) -> Option<u32> { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdlib_trait() {
        let stdlib = TestStdlib;
        let functions = stdlib.get_global_functions();

        assert_eq!(functions.len(), 2);
        assert_eq!(functions[0].0, "test_fn");
        assert_eq!(functions[1].0, "echo");

        let (name, version) = stdlib.info();
        assert_eq!(name, "test-stdlib");
        assert_eq!(version, "0.0.1");
    }

    #[test]
    fn test_method_registry_trait() {
        let registry = EmptyMethodRegistry;
        let mut pool = StringPool::new();

        assert!(!registry.has_method("string", "upper"));
        assert!(registry.call("string", "upper", &[], &mut pool).is_err());
        assert_eq!(registry.get_method_id("string", "upper"), None);
    }

    #[test]
    fn test_global_function_execution() {
        let stdlib = TestStdlib;
        let functions = stdlib.get_global_functions();
        let mut pool = StringPool::new();

        // Test test_fn
        let result = functions[0].1(&[], &mut pool);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Number(42.0));

        // Test echo with value
        let arg = Value::String(pool.intern("hello"));
        let result = functions[1].1(std::slice::from_ref(&arg), &mut pool);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), arg);

        // Test echo without value
        let result = functions[1].1(&[], &mut pool);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Null);
    }
}
