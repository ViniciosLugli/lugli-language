use hashbrown::HashMap;
use lugli_common::{LugliError, StringPool, Value};
use lugli_stdlib::MethodRegistry;

/// Handles method dispatch and caching for VM execution
pub struct InstructionDispatcher {
    /// Registry of methods for all types
    method_registry: MethodRegistry,

    /// Method call cache: (type_id, method_name_index) → method_hash
    /// This caches the hash computation to avoid repeated string operations
    method_cache: HashMap<(usize, usize), u32>,
}

impl InstructionDispatcher {
    pub fn new() -> Self {
        Self {
            method_registry: MethodRegistry::new(),
            method_cache: HashMap::with_capacity(256),
        }
    }

    /// Dispatch a method call using cached hash lookup
    ///
    /// This is the hot path for method calls. It:
    /// 1. Checks the cache for a precomputed hash
    /// 2. If cache miss, computes hash and caches it
    /// 3. Calls the method via the registry
    pub fn dispatch_cached(
        &mut self,
        type_id: usize,
        type_name: &str,
        method_name_index: usize,
        method_name: &str,
        args: &[Value],
        pool: &mut StringPool,
    ) -> Result<Value, LugliError> {
        let cache_key = (type_id, method_name_index);

        let hash = if let Some(&h) = self.method_cache.get(&cache_key) {
            h
        } else {
            let h = lugli_stdlib::methods::hash_method(type_name, method_name);
            self.method_cache.insert(cache_key, h);
            h
        };

        self.method_registry.call_by_hash(hash, args, pool)
    }

    /// Direct method call without caching (used for dynamic dispatch)
    pub fn dispatch(
        &self,
        type_name: &str,
        method_name: &str,
        args: &[Value],
        pool: &mut StringPool,
    ) -> Result<Value, LugliError> {
        self.method_registry.call(type_name, method_name, args, pool)
    }

    /// Check if a method exists for a type
    pub fn has_method(&self, type_name: &str, method_name: &str) -> bool {
        self.method_registry.has_method(type_name, method_name)
    }

    /// Clear the method cache (useful for testing or hot-reloading)
    pub fn clear_cache(&mut self) {
        self.method_cache.clear();
    }

    /// Get cache statistics
    pub fn cache_size(&self) -> usize {
        self.method_cache.len()
    }

    /// Get cache hit rate (requires tracking, not yet implemented)
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            size: self.method_cache.len(),
            capacity: self.method_cache.capacity(),
        }
    }
}

impl Default for InstructionDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub capacity: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatcher_creation() {
        let dispatcher = InstructionDispatcher::new();
        assert_eq!(dispatcher.cache_size(), 0);
    }

    #[test]
    fn test_has_method() {
        let dispatcher = InstructionDispatcher::new();

        // String methods should exist
        assert!(dispatcher.has_method("string", "upper"));
        assert!(dispatcher.has_method("string", "lower"));
        assert!(dispatcher.has_method("string", "len"));

        // List methods should exist
        assert!(dispatcher.has_method("list", "push"));
        assert!(dispatcher.has_method("list", "pop"));

        // Non-existent methods
        assert!(!dispatcher.has_method("string", "nonexistent"));
        assert!(!dispatcher.has_method("unknown_type", "method"));
    }

    #[test]
    fn test_cache_operations() {
        let mut dispatcher = InstructionDispatcher::new();
        assert_eq!(dispatcher.cache_size(), 0);

        // Simulate cache insertion (normally done by dispatch_cached)
        let type_id = 1;
        let method_idx = 42;
        let hash = lugli_stdlib::methods::hash_method("string", "upper");
        dispatcher.method_cache.insert((type_id, method_idx), hash);

        assert_eq!(dispatcher.cache_size(), 1);
        assert_eq!(dispatcher.method_cache.get(&(type_id, method_idx)), Some(&hash));

        dispatcher.clear_cache();
        assert_eq!(dispatcher.cache_size(), 0);
    }

    #[test]
    fn test_cache_stats() {
        let mut dispatcher = InstructionDispatcher::new();

        let stats = dispatcher.cache_stats();
        assert_eq!(stats.size, 0);
        assert!(stats.capacity >= 256); // Initial capacity

        // Add some cache entries
        dispatcher.method_cache.insert((1, 1), 100);
        dispatcher.method_cache.insert((2, 2), 200);

        let stats = dispatcher.cache_stats();
        assert_eq!(stats.size, 2);
    }

    #[test]
    fn test_dispatch_string_upper() {
        let dispatcher = InstructionDispatcher::new();
        let mut pool = StringPool::new();

        let string_val = Value::String(pool.intern("hello"));
        let args = vec![string_val];

        let result = dispatcher.dispatch("string", "upper", &args, &mut pool);
        assert!(result.is_ok());

        if let Ok(Value::String(id)) = result {
            assert_eq!(pool.resolve(id), "HELLO");
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_dispatch_list_len() {
        let dispatcher = InstructionDispatcher::new();
        let mut pool = StringPool::new();

        let list = Value::List(std::rc::Rc::new(std::cell::RefCell::new(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])));
        let args = vec![list];

        let result = dispatcher.dispatch("list", "len", &args, &mut pool);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Number(3.0));
    }

    #[test]
    fn test_dispatch_nonexistent_method() {
        let dispatcher = InstructionDispatcher::new();
        let mut pool = StringPool::new();

        let string_val = Value::String(pool.intern("test"));
        let args = vec![string_val];

        let result = dispatcher.dispatch("string", "nonexistent", &args, &mut pool);
        assert!(result.is_err());
    }
}
