use lugli_common::{LugliError, StringPool, Value};
use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
};

use crate::core::{dict, list, string};

pub type MethodFunction = fn(&[Value], &mut StringPool) -> Result<Value, LugliError>;

pub fn hash_method(type_name: &str, method: &str) -> u32 {
    let mut hasher = DefaultHasher::new();
    type_name.hash(&mut hasher);
    method.hash(&mut hasher);
    (hasher.finish() & 0xFFFFFFFF) as u32
}

pub struct MethodRegistry {
    methods: HashMap<(&'static str, &'static str), MethodFunction>,
    hash_methods: HashMap<u32, MethodFunction>,
    hash_names: HashMap<u32, (&'static str, &'static str)>,
}

impl MethodRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            methods: HashMap::new(),
            hash_methods: HashMap::new(),
            hash_names: HashMap::new(),
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
        self.register("dict", "has_key", dict::dict_contains); // Alias for contains
        self.register("dict", "get", dict::dict_get);
        self.register("dict", "len", dict::dict_len);
    }

    fn register(&mut self, type_name: &'static str, method: &'static str, func: MethodFunction) {
        self.methods.insert((type_name, method), func);
        let hash = hash_method(type_name, method);
        self.hash_methods.insert(hash, func);
        self.hash_names.insert(hash, (type_name, method));
    }

    pub fn call(&self, type_name: &str, method: &str, args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
        let func = self.methods.get(&(type_name, method)).ok_or_else(|| LugliError::runtime(format!("{} has no method '{}'", type_name, method)))?;

        func(args, pool)
    }

    pub fn call_by_hash(&self, hash: u32, args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
        let func = self.hash_methods.get(&hash).ok_or_else(|| {
            let name = self.hash_names.get(&hash).map(|(t, m)| format!("{}.{}", t, m)).unwrap_or_else(|| format!("method#{}", hash));
            LugliError::runtime(format!("{} not found", name))
        })?;

        func(args, pool)
    }

    pub fn has_method(&self, type_name: &str, method: &str) -> bool { self.methods.contains_key(&(type_name, method)) }

    pub fn has_method_hash(&self, hash: u32) -> bool { self.hash_methods.contains_key(&hash) }
}

impl Default for MethodRegistry {
    fn default() -> Self { Self::new() }
}

impl lugli_common::stdlib::MethodRegistry for MethodRegistry {
    fn call(
        &self,
        type_name: &str,
        method_name: &str,
        args: &[Value],
        pool: &mut StringPool,
    ) -> Result<Value, LugliError> {
        self.call(type_name, method_name, args, pool)
    }

    fn call_by_hash(
        &self,
        hash: u32,
        args: &[Value],
        pool: &mut StringPool,
    ) -> Result<Value, LugliError> {
        self.call_by_hash(hash, args, pool)
    }

    fn has_method(&self, type_name: &str, method_name: &str) -> bool {
        self.has_method(type_name, method_name)
    }

    fn get_method_id(&self, type_name: &str, method_name: &str) -> Option<u32> {
        Some(hash_method(type_name, method_name))
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

    #[test]
    fn test_hash_based_method_call() {
        let registry = MethodRegistry::new();
        let mut pool = StringPool::new();

        let hello_id = pool.intern("hello");
        let args = vec![Value::String(hello_id)];

        let hash = hash_method("string", "upper");
        let result = registry.call_by_hash(hash, &args, &mut pool).unwrap();

        if let Value::String(id) = result {
            assert_eq!(pool.resolve(id), "HELLO");
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_hash_method_not_found() {
        let registry = MethodRegistry::new();
        let mut pool = StringPool::new();

        let id = pool.intern("test");
        let args = vec![Value::String(id)];

        let invalid_hash = 999999;
        let result = registry.call_by_hash(invalid_hash, &args, &mut pool);
        assert!(result.is_err());
    }

    #[test]
    fn test_has_method_hash() {
        let registry = MethodRegistry::new();
        let hash = hash_method("string", "upper");
        assert!(registry.has_method_hash(hash));
        assert!(!registry.has_method_hash(999999));
    }
}
