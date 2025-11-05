use lugli_common::Span;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

impl NodeId {
    pub fn new(id: usize) -> Self {
        NodeId(id as u32)
    }
}

#[derive(Debug, Default, Clone)]
pub struct SpanMap {
    spans: HashMap<NodeId, Span>,
    next_id: u32,
}

impl SpanMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn alloc_id(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        id
    }

    pub fn insert(&mut self, id: NodeId, span: Span) {
        self.spans.insert(id, span);
    }

    pub fn get(&self, id: NodeId) -> Option<Span> {
        self.spans.get(&id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_map_allocation() {
        let mut map = SpanMap::new();
        let id1 = map.alloc_id();
        let id2 = map.alloc_id();

        assert_ne!(id1, id2);
        assert_eq!(id1.0 + 1, id2.0);
    }

    #[test]
    fn test_span_map_insert_get() {
        let mut map = SpanMap::new();
        let id = map.alloc_id();
        let span = Span { start: 0, end: 10 };

        map.insert(id, span);
        assert_eq!(map.get(id), Some(span));
    }

    #[test]
    fn test_span_map_missing_id() {
        let map = SpanMap::new();
        let id = NodeId(999);
        assert_eq!(map.get(id), None);
    }
}
