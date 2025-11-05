use std::{collections::HashMap, sync::Arc};

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
