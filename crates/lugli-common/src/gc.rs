use crate::Value;
use std::collections::HashSet;

/// Garbage collector for cycle detection inspired by Lua's incremental GC
/// and Ruby's tri-color marking algorithm.
///
/// Current implementation uses threshold-based triggering similar to Lua 5.4.
/// Future versions will implement full tri-color marking for better performance.
pub struct GarbageCollector {
    /// Allocation threshold before triggering GC (similar to Lua's GC pause)
    threshold: usize,

    /// Counter for allocations since last collection
    allocations_since_last_gc: usize,

    /// Growth factor for threshold adjustment (adaptive like Ruby's GC)
    growth_factor: f64,

    /// Minimum threshold to prevent thrashing
    min_threshold: usize,

    /// Maximum threshold to prevent memory bloat
    max_threshold: usize,

    /// Track if GC is currently running (for incremental collection)
    is_running: bool,
}

impl GarbageCollector {
    /// Create new GC with default settings optimized for general workloads
    pub fn new() -> Self {
        Self {
            threshold: 1000,
            allocations_since_last_gc: 0,
            growth_factor: 2.0,
            min_threshold: 100,
            max_threshold: 100_000,
            is_running: false,
        }
    }

    /// Create GC with custom threshold (for fine-tuning performance)
    pub fn with_threshold(threshold: usize) -> Self {
        Self {
            threshold: threshold.clamp(100, 100_000),
            allocations_since_last_gc: 0,
            growth_factor: 2.0,
            min_threshold: 100,
            max_threshold: 100_000,
            is_running: false,
        }
    }

    /// Record an allocation and trigger GC if threshold exceeded
    pub fn track_allocation(&mut self, roots: &[Value]) {
        self.allocations_since_last_gc += 1;

        if self.allocations_since_last_gc >= self.threshold {
            self.collect(roots);
        }
    }

    /// Manually trigger garbage collection
    pub fn collect(&mut self, roots: &[Value]) {
        if self.is_running {
            return; // Prevent recursive GC
        }

        self.is_running = true;
        let allocations_before_gc = self.allocations_since_last_gc;
        let marked = self.mark_reachable(roots);
        self.sweep(marked);
        self.adjust_threshold(allocations_before_gc);
        self.allocations_since_last_gc = 0;
        self.is_running = false;
    }

    /// Mark phase: Find all reachable values (tri-color marking)
    fn mark_reachable(&self, roots: &[Value]) -> HashSet<usize> {
        let mut marked = HashSet::new();
        let mut worklist = Vec::new();

        // Add roots to worklist
        for root in roots {
            worklist.push(root.clone());
        }

        // Process worklist until empty
        while let Some(value) = worklist.pop() {
            let id = self.value_id(&value);

            if marked.contains(&id) {
                continue;
            }

            marked.insert(id);

            // Add reachable values to worklist
            match &value {
                Value::Dict(dict) => {
                    if let Ok(dict_ref) = dict.try_borrow() {
                        for child in dict_ref.values() {
                            worklist.push(child.clone());
                        }
                    }
                }
                Value::List(list) => {
                    if let Ok(list_ref) = list.try_borrow() {
                        for child in list_ref.iter() {
                            worklist.push(child.clone());
                        }
                    }
                }
                Value::Closure {
                    upvalues, ..
                } => {
                    for upvalue in upvalues {
                        if let Ok(upvalue_ref) = upvalue.try_borrow() {
                            worklist.push(upvalue_ref.clone());
                        }
                    }
                }
                Value::StructInstance {
                    fields, ..
                } => {
                    for field in fields.values() {
                        worklist.push(field.clone());
                    }
                }
                _ => {}
            }
        }

        marked
    }

    /// Sweep phase: Currently a no-op as Rc automatically cleans up
    /// Future: Implement weak references or arena allocation
    fn sweep(&self, _marked: HashSet<usize>) {
        // Rust's Rc handles deallocation automatically
        // This is a placeholder for future arena-based allocation
    }

    /// Adaptive threshold adjustment (inspired by Ruby's GC tuning)
    fn adjust_threshold(&mut self, allocations_before_gc: usize) {
        if allocations_before_gc < self.threshold / 2 {
            // GC triggered early, reduce threshold
            self.threshold = (self.threshold as f64 / self.growth_factor) as usize;
        } else {
            // GC threshold was good, increase slightly
            self.threshold = (self.threshold as f64 * self.growth_factor) as usize;
        }

        self.threshold = self.threshold.clamp(self.min_threshold, self.max_threshold);
    }

    /// Generate unique ID for value (pointer-based hashing)
    ///
    /// NOTE: This is a best-effort approach for tracking values.
    /// - Dict/List: Stable pointer IDs work perfectly
    /// - Closure/Struct: Use value discriminant + hash of name/first field
    /// - Primitives: ID=0 (don't need tracking, no cycles possible)
    fn value_id(&self, value: &Value) -> usize {
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{Hash, Hasher},
        };

        match value {
            Value::Dict(d) => d.as_ptr() as usize,
            Value::List(l) => l.as_ptr() as usize,
            Value::Closure {
                name,
                upvalues,
                ..
            } => {
                // Combine closure name hash with upvalues pointer
                let mut hasher = DefaultHasher::new();
                name.hash(&mut hasher);
                upvalues.as_ptr().hash(&mut hasher);
                hasher.finish() as usize
            }
            Value::StructInstance {
                name,
                fields,
            } => {
                // Combine struct name with fields pointer
                let mut hasher = DefaultHasher::new();
                name.hash(&mut hasher);
                // Use the HashMap's address itself as stable identifier
                (fields as *const _ as usize).hash(&mut hasher);
                hasher.finish() as usize
            }
            // Primitives can't form cycles, skip tracking
            _ => 0,
        }
    }

    /// Detect cycles in value graph
    ///
    /// Returns descriptions of any cycles found
    pub fn detect_cycles(&self, roots: &[Value]) -> Vec<String> {
        let mut cycles = Vec::new();

        for (i, root) in roots.iter().enumerate() {
            if root.contains_cycle() {
                cycles.push(format!(
                    "Cycle detected in root value #{}: type={}",
                    i,
                    root.type_name()
                ));
            }
        }

        cycles
    }

    /// Get current GC statistics (for debugging and tuning)
    pub fn stats(&self) -> GCStats {
        GCStats {
            threshold: self.threshold,
            allocations: self.allocations_since_last_gc,
            is_running: self.is_running,
        }
    }
}

impl Default for GarbageCollector {
    fn default() -> Self { Self::new() }
}

/// GC statistics for monitoring and debugging
#[derive(Debug, Clone, Copy)]
pub struct GCStats {
    pub threshold: usize,
    pub allocations: usize,
    pub is_running: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_creation() {
        let gc = GarbageCollector::new();
        assert_eq!(gc.threshold, 1000);
        assert_eq!(gc.allocations_since_last_gc, 0);
    }

    #[test]
    fn test_gc_with_custom_threshold() {
        let gc = GarbageCollector::with_threshold(500);
        assert_eq!(gc.threshold, 500);
    }

    #[test]
    fn test_gc_threshold_bounds() {
        let gc = GarbageCollector::with_threshold(50);
        assert_eq!(gc.threshold, 100);

        let gc = GarbageCollector::with_threshold(200_000);
        assert_eq!(gc.threshold, 100_000);
    }

    #[test]
    fn test_allocation_tracking() {
        let mut gc = GarbageCollector::with_threshold(10);
        let roots = vec![];

        for _ in 0..5 {
            gc.track_allocation(&roots);
        }

        assert_eq!(gc.allocations_since_last_gc, 5);
    }
}
