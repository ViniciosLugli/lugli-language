mod helpers;
use helpers::run_test;
use lugli_common::GarbageCollector;

#[test]
fn test_circular_reference_warning() {
    let source = r#"
        let x = {}
        x.me = x
    "#;

    run_test(source);
}

#[test]
fn test_nested_circular_reference() {
    let source = r#"
        let a = {}
        let b = {}
        a.b = b
        b.a = a
    "#;

    run_test(source);
}

#[test]
fn test_list_circular_reference() {
    let source = r#"
        let x = []
        x.push(x)
    "#;

    run_test(source);
}

#[test]
fn test_struct_circular_reference() {
    let source = r#"
        struct Node {
            value
            next
        }

        let n1 = Node { value: 1, next: null }
        let n2 = Node { value: 2, next: n1 }
        n1.next = n2
    "#;

    run_test(source);
}

#[test]
fn test_gc_basic() {
    let gc = GarbageCollector::new();
    let stats = gc.stats();

    assert_eq!(stats.threshold, 1000);
    assert_eq!(stats.allocations, 0);
    assert!(!stats.is_running);
}

#[test]
fn test_gc_custom_threshold() {
    let gc = GarbageCollector::with_threshold(500);
    let stats = gc.stats();

    assert_eq!(stats.threshold, 500);
}

#[test]
fn test_gc_threshold_bounds() {
    let gc = GarbageCollector::with_threshold(50);
    assert_eq!(gc.stats().threshold, 100);

    let gc = GarbageCollector::with_threshold(200_000);
    assert_eq!(gc.stats().threshold, 100_000);
}

#[test]
fn test_deep_nesting_no_cycle() {
    let source = r#"
        let root = {}
        let level1 = {}
        let level2 = {}
        let level3 = {}

        root.child = level1
        level1.child = level2
        level2.child = level3
        level3.value = 42
    "#;

    run_test(source);
}

#[test]
fn test_multiple_references_no_cycle() {
    let source = r#"
        let shared = { value: 100 }
        let a = { ref: shared }
        let b = { ref: shared }
        let c = { ref: shared }
    "#;

    run_test(source);
}

#[test]
fn test_empty_struct_no_crash() {
    let source = r#"
        struct Empty {}
        let e1 = Empty {}
        let e2 = Empty {}
    "#;

    run_test(source);
}

#[test]
fn test_deeply_nested_lists() {
    let source = r#"
        let root = []
        let level1 = []
        let level2 = []
        let level3 = []

        root.push(level1)
        level1.push(level2)
        level2.push(level3)
        level3.push(42)
    "#;

    run_test(source);
}

#[test]
fn test_mixed_collection_types() {
    let source = r#"
        let dict = { items: [] }
        let list = [dict]
        dict.items.push(list)
    "#;

    run_test(source);
}

#[test]
fn test_large_allocation_pattern() {
    let source = r#"
        let items = []
        let i = 0
        while i < 50 {
            items.push({ id: i, data: [] })
            i = i + 1
        }
    "#;

    run_test(source);
}

#[test]
fn test_gc_adaptive_threshold() {
    let mut gc = GarbageCollector::with_threshold(200);
    let roots = vec![];

    let initial_threshold = gc.stats().threshold;
    assert_eq!(initial_threshold, 200);

    // Fill up to threshold (should trigger GC and adjust)
    for _ in 0..200 {
        gc.track_allocation(&roots);
    }

    // After GC, threshold should have increased (allocations hit threshold)
    let new_threshold = gc.stats().threshold;
    assert!(new_threshold >= initial_threshold, "Threshold adjusted: {} -> {}", initial_threshold, new_threshold);

    // Verify GC was triggered (allocations reset)
    assert_eq!(gc.stats().allocations, 0, "Allocations should reset after GC");
}

#[test]
fn test_gc_prevents_thrashing() {
    let gc = GarbageCollector::with_threshold(50);
    let stats = gc.stats();

    assert!(stats.threshold >= 100, "Threshold should be at least min_threshold");
}

#[test]
fn test_gc_prevents_bloat() {
    let gc = GarbageCollector::with_threshold(500_000);
    let stats = gc.stats();

    assert!(stats.threshold <= 100_000, "Threshold should be at most max_threshold");
}
