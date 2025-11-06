use hashbrown::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StringId(u32);

impl StringId {
    pub fn as_u32(self) -> u32 { self.0 }
}

#[derive(Debug, Clone)]
pub struct StringPool {
    strings: Vec<Arc<str>>,
    lookup: HashMap<Arc<str>, StringId>,
}

impl StringPool {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            lookup: HashMap::new(),
        }
    }

    pub fn with_common_strings() -> Self {
        let mut pool = Self {
            strings: Vec::with_capacity(64),
            lookup: HashMap::with_capacity(64),
        };

        // Pre-intern most common identifiers and keywords
        let common = [
            "self", "true", "false", "null", "len", "push", "pop", "get", "set",
            "name", "value", "result", "data", "error", "message", "type", "item",
            "index", "key", "size", "count", "list", "dict", "string", "number",
            "add", "remove", "contains", "clear", "empty", "start", "end", "text",
            "i", "j", "k", "x", "y", "z", "a", "b", "c", "n", "m", "tmp", "temp",
            "main", "fn", "struct", "if", "else", "for", "while", "return", "match",
            "print", "input", "str", "int", "float", "bool", "upper", "lower", "split",
        ];

        for s in &common {
            pool.intern(s);
        }

        pool
    }

    pub fn intern(&mut self, s: &str) -> StringId {
        if let Some(&id) = self.lookup.get(s) {
            return id;
        }

        let id = StringId(self.strings.len() as u32);
        let arc_str: Arc<str> = Arc::from(s);
        self.strings.push(arc_str.clone());
        self.lookup.insert(arc_str, id);
        id
    }

    pub fn resolve(&self, id: StringId) -> &str { &self.strings[id.0 as usize] }

    pub fn try_resolve(&self, id: StringId) -> Option<&str> { self.strings.get(id.0 as usize).map(|s| s.as_ref()) }

    pub fn len(&self) -> usize { self.strings.len() }

    pub fn is_empty(&self) -> bool { self.strings.is_empty() }
}

impl Default for StringPool {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_interning() {
        let mut pool = StringPool::new();
        let id1 = pool.intern("hello");
        let id2 = pool.intern("hello");
        assert_eq!(id1, id2);

        let id3 = pool.intern("world");
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_resolve() {
        let mut pool = StringPool::new();
        let id = pool.intern("test");
        assert_eq!(pool.resolve(id), "test");
    }

    #[test]
    fn test_duplicate_allocations_reduced() {
        let mut pool = StringPool::new();
        pool.intern("duplicate");
        pool.intern("duplicate");
        pool.intern("duplicate");

        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_multiple_strings() {
        let mut pool = StringPool::new();
        let id1 = pool.intern("first");
        let id2 = pool.intern("second");
        let id3 = pool.intern("first");

        assert_eq!(id1, id3);
        assert_ne!(id1, id2);
        assert_eq!(pool.len(), 2);
    }
}
