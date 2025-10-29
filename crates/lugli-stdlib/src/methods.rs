use lugli_common::{LugliError, StringPool, Value};
use std::collections::HashMap;

use crate::core::{dict, list, string};

pub type MethodFunction = fn(&[Value], &mut StringPool) -> Result<Value, LugliError>;

pub struct MethodRegistry {
    methods: HashMap<(&'static str, &'static str), MethodFunction>,
}

impl MethodRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            methods: HashMap::new(),
        };
        registry.register_all();
        registry
    }

    fn register_all(&mut self) {
        // String methods
        self.register("string", "len", string::string_length);
        self.register("string", "trim", string::string_trim);
        self.register("string", "lower", string::string_lower);
        self.register("string", "upper", string::string_upper);
        self.register("string", "chars", string::string_chars);
        self.register("string", "split", string::string_split);
        self.register("string", "is_alphabetic", string::string_is_alphabetic);
        self.register("string", "starts_with", string::string_starts_with);
        self.register("string", "ends_with", string::string_ends_with);
        self.register("string", "contains", string::string_contains);
        self.register("string", "replace", string::string_replace);

        // List methods
        self.register("list", "len", list::list_length);
        self.register("list", "push", list::list_push);
        self.register("list", "pop", list::list_pop);
        self.register("list", "join", list::list_join);
        self.register("list", "contains", list::list_contains);
        self.register("list", "is_empty", list::list_is_empty);
        self.register("list", "clear", list::list_clear);
        self.register("list", "reverse", list::list_reverse);
        self.register("list", "append", list::list_append);
        self.register("list", "get", list::list_get);
        self.register("list", "set", list::list_set);
        self.register("list", "sorted", list::list_sorted);
        self.register("list", "reversed", list::list_reversed);
        // Note: map and filter are handled specially in VM for higher-order functions

        // Dict methods
        self.register("dict", "keys", dict::dict_keys);
        self.register("dict", "values", dict::dict_values);
        self.register("dict", "contains", dict::dict_contains);
        self.register("dict", "get", dict::dict_get);
    }

    fn register(&mut self, type_name: &'static str, method: &'static str, func: MethodFunction) {
        self.methods.insert((type_name, method), func);
    }

    pub fn call(&self, type_name: &str, method: &str, args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
        let func = self
            .methods
            .get(&(type_name, method))
            .ok_or_else(|| LugliError::runtime(format!("{} has no method '{}'", type_name, method)))?;

        func(args, pool)
    }

    pub fn has_method(&self, type_name: &str, method: &str) -> bool {
        self.methods.contains_key(&(type_name, method))
    }
}

impl Default for MethodRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_registry_creation() {
        let registry = MethodRegistry::new();
        assert!(registry.has_method("string", "upper"));
        assert!(registry.has_method("list", "push"));
        assert!(!registry.has_method("string", "nonexistent"));
    }

    #[test]
    fn test_string_method_call() {
        let registry = MethodRegistry::new();
        let mut pool = StringPool::new();

        let hello_id = pool.intern("hello");
        let args = vec![Value::String(hello_id)];

        let result = registry.call("string", "upper", &args, &mut pool).unwrap();

        if let Value::String(id) = result {
            assert_eq!(pool.resolve(id), "HELLO");
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_method_not_found() {
        let registry = MethodRegistry::new();
        let mut pool = StringPool::new();

        let id = pool.intern("test");
        let args = vec![Value::String(id)];

        let result = registry.call("string", "nonexistent", &args, &mut pool);
        assert!(result.is_err());
    }
}
