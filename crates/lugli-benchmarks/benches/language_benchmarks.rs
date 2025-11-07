use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::{fs, hint::black_box, time::Duration};

// Helper function to run a Lugli source file
fn run_lugli_source(source: &str) -> Result<lugli_common::Value, String> {
    let (program, span_map) = lugli_parser::parse(source).map_err(|e| e.to_string())?;
    let mut vm = lugli_vm::Vm::new();
    vm.compile_and_run(&program, span_map).map_err(|e| e.to_string())
}

// Helper function to compile Lugli source
fn compile_lugli_source(source: &str) -> Result<lugli_vm::Bytecode, String> {
    let (program, span_map) = lugli_parser::parse(source).map_err(|e| e.to_string())?;
    let vm = lugli_vm::Vm::new();
    vm.compile(&program, span_map).map_err(|e| e.to_string())
}

// Helper function to parse Lugli source
fn parse_lugli_source(source: &str) -> Result<(lugli_ast::Program, lugli_ast::SpanMap), String> {
    lugli_parser::parse(source).map_err(|e| e.to_string())
}

// Helper to load fixture (from crate's benches/fixtures/)
fn load_fixture(name: &str) -> String {
    let path = format!("{}/benches/fixtures/{}.lg", env!("CARGO_MANIFEST_DIR"), name);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to load fixture: {} (path: {})", name, path))
}

// Helper to load example from workspace root
fn load_example(relative_path: &str) -> String {
    // CARGO_MANIFEST_DIR is crates/lugli-benchmarks, so go up 2 levels to workspace root
    let workspace_root = format!("{}/../../", env!("CARGO_MANIFEST_DIR"));
    let path = format!("{}{}", workspace_root, relative_path);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to load example: {} (path: {})", relative_path, path))
}

// ========================================
// 1. STARTUP & HELLO WORLD
// ========================================

fn bench_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(3));

    let hello_world = load_fixture("hello_world");

    group.bench_function("cold_start_hello_world", |b| {
        b.iter(|| {
            run_lugli_source(black_box(&hello_world)).unwrap();
        });
    });

    group.finish();
}

// ========================================
// 2. ALGORITHMS
// ========================================

fn bench_algorithms(c: &mut Criterion) {
    let mut group = c.benchmark_group("algorithms");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(5));

    // Fibonacci
    let fib_source = load_fixture("fibonacci");
    group.bench_function("fibonacci_recursive_n20", |b| {
        b.iter(|| {
            run_lugli_source(black_box(&fib_source)).unwrap();
        });
    });

    // Primes
    let primes_source = load_fixture("primes");
    group.bench_function("primes_up_to_1000", |b| {
        b.iter(|| {
            run_lugli_source(black_box(&primes_source)).unwrap();
        });
    });

    // Sorting
    let sorting_source = load_fixture("sorting");
    group.bench_function("quicksort_15_elements", |b| {
        b.iter(|| {
            run_lugli_source(black_box(&sorting_source)).unwrap();
        });
    });

    group.finish();
}

// ========================================
// 3. DATA PROCESSING
// ========================================

fn bench_data_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_processing");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(5));

    let data_source = load_fixture("data_processing");

    for size in [100, 500, 1000].iter() {
        let source = data_source.replace("process_data(1000)", &format!("process_data({})", size));
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &source, |b, src| {
            b.iter(|| {
                run_lugli_source(black_box(src)).unwrap();
            });
        });
    }

    group.finish();
}

// ========================================
// 4. REAL PROGRAMS
// ========================================

fn bench_real_programs(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_programs");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(5));

    // Pattern matching (110 lines)
    let pattern_source = load_example("examples/basics/07_pattern_matching.lg");
    group.bench_function("pattern_matching_110_lines", |b| {
        b.iter(|| {
            run_lugli_source(black_box(&pattern_source)).unwrap();
        });
    });

    // List comprehensions (179 lines)
    let list_comp_source = load_example("examples/basics/08_list_comprehensions.lg");
    group.bench_function("list_comprehensions_179_lines", |b| {
        b.iter(|| {
            run_lugli_source(black_box(&list_comp_source)).unwrap();
        });
    });

    // Calculator
    let calculator_source = load_example("examples/basics/05_calculator.lg");
    group.bench_function("calculator", |b| {
        b.iter(|| {
            run_lugli_source(black_box(&calculator_source)).unwrap();
        });
    });

    // Advanced features
    let advanced_source = load_example("examples/basics/06_advanced_features.lg");
    group.bench_function("advanced_features_60_lines", |b| {
        b.iter(|| {
            run_lugli_source(black_box(&advanced_source)).unwrap();
        });
    });

    group.finish();
}

// ========================================
// 5. LANGUAGE FEATURES
// ========================================

fn bench_language_features(c: &mut Criterion) {
    let mut group = c.benchmark_group("language_features");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(5));

    // Closures
    let closures_source = load_fixture("closures");
    group.bench_function("closures_500_iterations", |b| {
        b.iter(|| {
            run_lugli_source(black_box(&closures_source)).unwrap();
        });
    });

    // Pattern matching
    let pattern_source = load_fixture("pattern_matching");
    group.bench_function("pattern_matching_1000_iterations", |b| {
        b.iter(|| {
            run_lugli_source(black_box(&pattern_source)).unwrap();
        });
    });

    // Structs
    let structs_source = load_fixture("structs");
    group.bench_function("structs_500_instances", |b| {
        b.iter(|| {
            run_lugli_source(black_box(&structs_source)).unwrap();
        });
    });

    group.finish();
}

// ========================================
// 6. COMMON PATTERNS
// ========================================

fn bench_common_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("common_patterns");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(5));

    // Nested loops
    let nested_source = load_fixture("nested_loops");
    group.bench_function("nested_loops_50x50x10", |b| {
        b.iter(|| {
            run_lugli_source(black_box(&nested_source)).unwrap();
        });
    });

    // String operations
    let string_source = load_fixture("string_operations");
    group.bench_function("string_operations_500_iterations", |b| {
        b.iter(|| {
            run_lugli_source(black_box(&string_source)).unwrap();
        });
    });

    group.finish();
}

// ========================================
// 7. COLLECTIONS PERFORMANCE
// ========================================

fn bench_collections(c: &mut Criterion) {
    let mut group = c.benchmark_group("collections");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(5));

    // List operations - push
    let list_push = r#"
fn bench_list_push(n) {
    let list = []
    mut i = 0
    while i < n {
        list.push(i)
        i = i + 1
    }
    return list
}
bench_list_push(1000)
"#;

    group.throughput(Throughput::Elements(1000));
    group.bench_function("list_push_1000_items", |b| {
        b.iter(|| {
            run_lugli_source(black_box(list_push)).unwrap();
        });
    });

    // List comprehension
    let list_comp = r#"
let numbers = [x for x in range(1000)]
let evens = [x for x in numbers if x % 2 == 0]
evens
"#;

    group.bench_function("list_comprehension_1000_items", |b| {
        b.iter(|| {
            run_lugli_source(black_box(list_comp)).unwrap();
        });
    });

    // Dictionary operations
    let dict_ops = r#"
fn bench_dict(n) {
    let dict = {}
    mut i = 0
    while i < n {
        dict[i] = i * 2
        i = i + 1
    }

    mut sum = 0
    mut j = 0
    while j < n {
        sum = sum + dict[j]
        j = j + 1
    }
    return sum
}
bench_dict(500)
"#;

    group.throughput(Throughput::Elements(500));
    group.bench_function("dict_insert_lookup_500_items", |b| {
        b.iter(|| {
            run_lugli_source(black_box(dict_ops)).unwrap();
        });
    });

    group.finish();
}

// ========================================
// 8. CONTROL FLOW
// ========================================

fn bench_control_flow(c: &mut Criterion) {
    let mut group = c.benchmark_group("control_flow");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(5));

    // If/elif/else chains
    let if_chain = r#"
fn classify(n) {
    if n < 10 {
        return "small"
    } elif n < 50 {
        return "medium"
    } elif n < 100 {
        return "large"
    } else {
        return "huge"
    }
}

mut i = 0
while i < 1000 {
    classify(i)
    i = i + 1
}
"#;

    group.bench_function("if_elif_chain_1000_iterations", |b| {
        b.iter(|| {
            run_lugli_source(black_box(if_chain)).unwrap();
        });
    });

    // For loops
    let for_loop = r#"
mut sum = 0
for i in range(1000) {
    sum = sum + i
}
sum
"#;

    group.bench_function("for_loop_1000_iterations", |b| {
        b.iter(|| {
            run_lugli_source(black_box(for_loop)).unwrap();
        });
    });

    // While loops
    let while_loop = r#"
mut sum = 0
mut i = 0
while i < 1000 {
    sum = sum + i
    i = i + 1
}
sum
"#;

    group.bench_function("while_loop_1000_iterations", |b| {
        b.iter(|| {
            run_lugli_source(black_box(while_loop)).unwrap();
        });
    });

    group.finish();
}

// ========================================
// 9. COMPILATION SPEED
// ========================================

fn bench_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilation");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(5));

    // Small program - hello world
    let small = load_fixture("hello_world");
    group.throughput(Throughput::Bytes(small.len() as u64));
    group.bench_function("parse_hello_world", |b| {
        b.iter(|| {
            parse_lugli_source(black_box(&small)).unwrap();
        });
    });

    group.bench_function("compile_hello_world", |b| {
        b.iter(|| {
            compile_lugli_source(black_box(&small)).unwrap();
        });
    });

    // Medium program - pattern matching
    let medium = load_example("examples/basics/07_pattern_matching.lg");
    group.throughput(Throughput::Bytes(medium.len() as u64));
    group.bench_function("parse_pattern_matching_110_lines", |b| {
        b.iter(|| {
            parse_lugli_source(black_box(&medium)).unwrap();
        });
    });

    group.bench_function("compile_pattern_matching_110_lines", |b| {
        b.iter(|| {
            compile_lugli_source(black_box(&medium)).unwrap();
        });
    });

    // Large program - list comprehensions
    let large = load_example("examples/basics/08_list_comprehensions.lg");
    group.throughput(Throughput::Bytes(large.len() as u64));
    group.bench_function("parse_list_comprehensions_179_lines", |b| {
        b.iter(|| {
            parse_lugli_source(black_box(&large)).unwrap();
        });
    });

    group.bench_function("compile_list_comprehensions_179_lines", |b| {
        b.iter(|| {
            compile_lugli_source(black_box(&large)).unwrap();
        });
    });

    group.finish();
}

// ========================================
// 10. THROUGHPUT METRICS
// ========================================

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(5));

    // Instructions per second - arithmetic heavy
    let arithmetic = r#"
mut sum = 0
mut i = 0
while i < 10000 {
    sum = sum + i * 2 - 1
    i = i + 1
}
sum
"#;

    group.bench_function("arithmetic_operations_10k", |b| {
        b.iter(|| {
            run_lugli_source(black_box(arithmetic)).unwrap();
        });
    });

    // Function call throughput
    let func_calls = r#"
fn add(a, b) {
    return a + b
}

mut sum = 0
mut i = 0
while i < 1000 {
    sum = add(sum, i)
    i = i + 1
}
sum
"#;

    group.bench_function("function_calls_1000", |b| {
        b.iter(|| {
            run_lugli_source(black_box(func_calls)).unwrap();
        });
    });

    group.finish();
}

// ========================================
// CRITERION CONFIGURATION
// ========================================

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(5));
    targets =
        bench_startup,
        bench_algorithms,
        bench_data_processing,
        bench_real_programs,
        bench_language_features,
        bench_common_patterns,
        bench_collections,
        bench_control_flow,
        bench_compilation,
        bench_throughput
}

criterion_main!(benches);
