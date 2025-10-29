# Lugli Language Performance Benchmarks

User-facing language performance benchmarks for Lugli. These benchmarks measure what users care about: **how fast their programs run**.

## Quick Start

```bash
# Run all benchmarks (~2-3 minutes)
just bench

# Quick benchmarks for CI (~30 seconds)
just bench-quick

# Run specific benchmark group
just bench-group algorithms

# Open HTML report
just bench-report

# Save baseline for comparison
just bench-baseline v0.4.0

# Compare against baseline
just bench-compare v0.4.0
```

## Benchmark Categories

### 1. Startup & Hello World
- **cold_start_hello_world**: Complete startup time for minimal program
- **Target**: <100ms startup time

### 2. Algorithms
- **fibonacci_recursive_n20**: Classic recursive Fibonacci
- **primes_up_to_1000**: Prime number generation using trial division
- **quicksort_15_elements**: Quicksort with list comprehensions and concatenation

### 3. Data Processing
- Parameterized benchmarks at 100, 500, 1000 items
- List operations, comprehensions with filters
- Dictionary operations
- String manipulations

### 4. Real Programs
- **pattern_matching_110_lines**: Comprehensive pattern matching examples
- **list_comprehensions_179_lines**: Advanced list comprehension showcase
- **calculator**: Full calculator implementation
- **advanced_features_60_lines**: Advanced language features demo

### 5. Language Features
- **closures_500_iterations**: Closure creation and execution
- **pattern_matching_1000_iterations**: Match statements with guards
- **structs_500_instances**: Struct instantiation and method calls

### 6. Common Patterns
- **nested_loops_50x50x10**: Triple-nested loop performance
- **string_operations_500_iterations**: String methods (upper, lower, split, etc.)

### 7. Collections Performance
- **list_push_1000_items**: List append performance
- **list_comprehension_1000_items**: List comprehension throughput
- **dict_insert_lookup_500_items**: Dictionary operations

### 8. Control Flow
- **if_elif_chain_1000_iterations**: Branching overhead
- **for_loop_1000_iterations**: For loop performance
- **while_loop_1000_iterations**: While loop performance

### 9. Compilation Speed
- **parse_hello_world**: Lexer + Parser for small program
- **compile_hello_world**: Full compilation for small program
- **parse_hangman_129_lines**: Parser throughput (MB/s)
- **compile_hangman_129_lines**: Compiler throughput
- **parse_list_comprehensions_179_lines**: Large program parsing
- **compile_list_comprehensions_179_lines**: Large program compilation

### 10. Throughput Metrics
- **arithmetic_operations_10k**: VM instruction throughput
- **function_calls_1000**: Function call overhead

## Performance Targets

From `CLAUDE.md` documentation:

| Component | Target | Status |
|-----------|--------|--------|
| Lexer | >1MB/s tokenization | ✓ Tested |
| Parser | >500KB/s parsing | ✓ Tested |
| VM Execution | >1M instructions/sec | ✓ Tested |
| Startup Time | <100ms cold start | ✓ Tested |
| Memory Usage | <10MB for basic programs | ⚠️ Not measured yet |

## Benchmark Structure

```
crates/lugli-benchmarks/
├── benches/
│   ├── language_benchmarks.rs  # Main benchmark suite
│   └── fixtures/               # Benchmark programs
│       ├── fibonacci.lg
│       ├── primes.lg
│       ├── sorting.lg
│       ├── data_processing.lg
│       ├── nested_loops.lg
│       ├── pattern_matching.lg
│       ├── structs.lg
│       ├── closures.lg
│       ├── string_operations.lg
│       └── hello_world.lg
├── src/
│   └── lib.rs                  # Placeholder
├── Cargo.toml
└── README.md
```

## Reading Results

Criterion outputs:
- **Time**: Mean execution time with confidence interval
- **Throughput**: Operations per second (where applicable)
- **Change**: Comparison to previous run (if available)
- **Outliers**: Statistical anomalies detected

Example output:
```
startup/cold_start_hello_world
                        time:   [14.410 µs 14.707 µs 15.104 µs]
Found 4 outliers among 100 measurements (4.00%)
```

## HTML Reports

After running benchmarks, open the detailed HTML report:
```bash
just bench-report
# Or manually: open target/criterion/report/index.html
```

The report includes:
- Violin plots showing distribution
- Line charts comparing runs
- Statistical analysis
- Regression detection

## CI Integration

For continuous performance monitoring:

```yaml
# .github/workflows/benchmarks.yml
- name: Run benchmarks
  run: just bench-quick

- name: Compare to main
  run: |
    git checkout main
    just bench-baseline main
    git checkout -
    just bench-compare main
```

## Adding New Benchmarks

1. **Create fixture** (if needed):
   ```lugli
   # benches/fixtures/my_feature.lg
   fn my_feature() {
       # Implementation
   }
   my_feature()
   ```

2. **Add benchmark function**:
   ```rust
   fn bench_my_feature(c: &mut Criterion) {
       let mut group = c.benchmark_group("my_category");
       let source = load_fixture("my_feature");

       group.bench_function("my_feature_description", |b| {
           b.iter(|| {
               run_lugli_source(black_box(&source)).unwrap();
           });
       });

       group.finish();
   }
   ```

3. **Register in criterion_group**:
   ```rust
   criterion_group! {
       name = benches;
       config = Criterion::default()...
       targets = ..., bench_my_feature  // Add here
   }
   ```

## Profiling Integration

Generate flamegraphs for deep analysis:
```bash
cargo install flamegraph
cargo flamegraph --bench language_benchmarks -- --bench startup
```

## Notes

- Benchmarks use release mode for accurate measurements
- Each benchmark warms up for 500ms before measurement
- Sample size: 100 (configurable via criterion config)
- Measurement time: 5 seconds per benchmark
- Using `std::hint::black_box()` to prevent compiler optimization
- Paths resolved via `CARGO_MANIFEST_DIR` for workspace compatibility

## Version

Current benchmark suite: **v1.0.0**
Last updated: **2025-10-29**
