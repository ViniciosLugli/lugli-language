# Phase 3: Performance Optimization (3-4 weeks)

**Goal:** Address performance bottlenecks, enable optimizer features

**Priority:** P1 - Performance improvements

**Dependencies:** Phase 2 must be complete

**Success Criteria:**
- ✅ 10-20% bytecode size reduction
- ✅ 100MB+ memory savings on module-heavy programs
- ✅ All optimizer features enabled
- ✅ Sub-100ms startup time maintained
- ✅ >1M instructions/second maintained

---

## Week 10-11: Memory Optimization

### Task 3.1: Replace HashMap Clones with Rc Wrappers (4 days)

**Priority:** CRITICAL
**Files:**
- `crates/lugli-vm/src/machine/context.rs`
- `crates/lugli-vm/src/machine/module_runtime.rs`
- `crates/lugli-vm/src/machine/mod.rs:461-625`

**Current Problem:**

Module loading clones entire globals HashMap:
```rust
// Line 461
let saved_globals = self.globals.clone();  // Clones 100+ entries!

// Line 605
let mut module_exports = self.globals.clone();  // Another full clone

// Line 625
self.module_globals.insert(module_bytecode_id, Rc::new(module_exports.clone()));  // Third clone!
```

**Impact:**
- 3 full HashMap clones per module import
- O(n) complexity for each clone where n = number of globals
- ~100MB wasted on large projects with many modules

**Solution: Copy-on-Write Globals**

**Update ExecutionContext:**

```rust
use std::rc::Rc;
use std::cell::RefCell;

pub struct ExecutionContext {
    stack: Vec<Value>,

    /// Shared globals with copy-on-write semantics
    globals: Rc<RefCell<HashMap<String, Value>>>,

    call_stack: Vec<CallFrame>,
    open_upvalues: HashMap<usize, Rc<RefCell<Value>>>,
    ip: usize,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            globals: Rc::new(RefCell::new(HashMap::new())),
            call_stack: Vec::new(),
            open_upvalues: HashMap::new(),
            ip: 0,
        }
    }

    /// Clone for module isolation
    pub fn snapshot_globals(&self) -> Rc<RefCell<HashMap<String, Value>>> {
        Rc::clone(&self.globals)  // O(1) operation instead of O(n)
    }

    /// Restore globals from snapshot
    pub fn restore_globals(&mut self, snapshot: Rc<RefCell<HashMap<String, Value>>>) {
        self.globals = snapshot;  // O(1) operation
    }

    /// Get global (read-only access)
    pub fn get_global(&self, name: &str) -> Option<Value> {
        self.globals.borrow().get(name).cloned()
    }

    /// Set global (handles copy-on-write)
    pub fn set_global(&mut self, name: String, value: Value) {
        // If Rc has multiple owners, make a copy before mutating
        if Rc::strong_count(&self.globals) > 1 {
            let new_globals = self.globals.borrow().clone();
            self.globals = Rc::new(RefCell::new(new_globals));
        }

        self.globals.borrow_mut().insert(name, value);
    }

    /// Define new global
    pub fn define_global(&mut self, name: String, value: Value) {
        // Same COW logic
        if Rc::strong_count(&self.globals) > 1 {
            let new_globals = self.globals.borrow().clone();
            self.globals = Rc::new(RefCell::new(new_globals));
        }

        self.globals.borrow_mut().insert(name, value);
    }
}
```

**Update Module Loading:**

```rust
// In machine/mod.rs
pub(super) fn exec_import_module(&mut self, module_path_id: usize) -> Result<(), LugliError> {
    let module_path = self.get_string(module_path_id)?;

    // Save current globals (O(1) - just Rc clone)
    let saved_globals = self.context.snapshot_globals();

    // ... module loading logic ...

    // Module execution with isolated globals
    self.run_module(&module_bytecode)?;

    // Save module exports (O(1) - just Rc clone)
    let module_exports = self.context.snapshot_globals();
    self.modules.save_module_globals(module_bytecode_id, module_exports);

    // Restore original globals (O(1) - just swap Rc)
    self.context.restore_globals(saved_globals);

    Ok(())
}
```

**Performance Measurement:**

Before:
```rust
#[test]
fn bench_module_imports_before() {
    // 100 globals, 10 module imports
    // Expected: ~3 full HashMap clones × 10 imports = 30 clones
    // Time: ~500ms for 10 imports
}
```

After:
```rust
#[test]
fn bench_module_imports_after() {
    // 100 globals, 10 module imports
    // Expected: Rc clones only (O(1))
    // Time: <50ms for 10 imports (10x improvement)
}
```

**Testing:**

```rust
#[test]
fn test_cow_globals_isolation() {
    let mut ctx = ExecutionContext::new();
    ctx.define_global("x".to_string(), Value::Number(10.0));

    // Snapshot
    let snapshot = ctx.snapshot_globals();

    // Modify
    ctx.set_global("x".to_string(), Value::Number(20.0));

    // Snapshot should be unchanged
    assert_eq!(snapshot.borrow().get("x"), Some(&Value::Number(10.0)));
    assert_eq!(ctx.get_global("x"), Some(Value::Number(20.0)));
}

#[test]
fn test_cow_triggers_copy() {
    let mut ctx = ExecutionContext::new();
    ctx.define_global("x".to_string(), Value::Number(10.0));

    let snapshot = ctx.snapshot_globals();

    // This should trigger COW
    ctx.set_global("x".to_string(), Value::Number(20.0));

    // Strong count should be 1 again for ctx.globals
    // (snapshot holds the old one)
    assert_eq!(Rc::strong_count(&ctx.globals), 1);
}
```

**Success criteria:**
- Module imports 10x faster
- Memory usage reduced by 100MB+ on module-heavy programs
- All module tests pass
- No regression in functionality

---

### Task 3.2: Reduce Clone Operations in Compiler (3 days)

**Priority:** HIGH
**Files:**
- `crates/lugli-vm/src/compiler/statements.rs:139-141, 160-163, 167`
- `crates/lugli-vm/src/machine/object_ops.rs:150-172`

**Current Problem:**

Function compilation clones locals and upvalues:
```rust
// Line 160-163 in statements.rs
let closure_value = Value::Closure {
    function: func_ref.clone(),  // Clones entire function metadata
    upvalues: upvalues.clone(),  // Clones entire upvalue list
    bytecode_id: Some(func_bytecode_id),
};
```

Upvalue capture clones stack values:
```rust
// Line in object_ops.rs
let value = self.stack.get(absolute_index).cloned()  // Unnecessary clone
    .ok_or_else(...)?;
let shared = Rc::new(RefCell::new(value));
```

**Solution 1: Intern Function Metadata**

```rust
// Create a FunctionRegistry
pub struct FunctionRegistry {
    functions: Vec<Rc<FunctionMetadata>>,
    next_id: usize,
}

impl FunctionRegistry {
    pub fn register(&mut self, metadata: FunctionMetadata) -> FunctionId {
        let id = self.next_id;
        self.next_id += 1;
        self.functions.push(Rc::new(metadata));
        FunctionId(id)
    }

    pub fn get(&self, id: FunctionId) -> Option<&Rc<FunctionMetadata>> {
        self.functions.get(id.0)
    }
}

// Instead of cloning metadata, just clone the Rc
let closure_value = Value::Closure {
    function_id: func_id,  // Just an ID (usize)
    upvalues: upvalues,    // This still needs to be cloned
    bytecode_id: Some(func_bytecode_id),
};
```

**Solution 2: Smart Upvalue Capture**

```rust
pub(super) fn capture_upvalue(&mut self, absolute_index: usize) -> Result<Rc<RefCell<Value>>, LugliError> {
    if let Some(existing) = self.open_upvalues.get(&absolute_index) {
        return Ok(Rc::clone(existing));  // Already captured, reuse
    }

    // Don't clone the value - wrap it directly
    let value = self.stack.get(absolute_index)
        .ok_or_else(|| LugliError::runtime("Invalid stack index"))?;

    // Create upvalue that references the value
    // This requires changing how upvalues work - they should hold indices
    // until closure, then copy values

    let upvalue = Rc::new(RefCell::new(value.clone()));
    self.open_upvalues.insert(absolute_index, Rc::clone(&upvalue));
    Ok(upvalue)
}
```

**Better Solution: Delay Cloning**

```rust
pub enum Upvalue {
    Open(usize),           // Stack index (not yet closed)
    Closed(Rc<RefCell<Value>>),  // Closed over value
}

pub(super) fn capture_upvalue(&mut self, absolute_index: usize) -> Result<Rc<Upvalue>, LugliError> {
    if let Some(existing) = self.open_upvalues.get(&absolute_index) {
        return Ok(Rc::clone(existing));
    }

    // Just store the index, no cloning yet
    let upvalue = Rc::new(Upvalue::Open(absolute_index));
    self.open_upvalues.insert(absolute_index, Rc::clone(&upvalue));
    Ok(upvalue)
}

pub(super) fn close_upvalues(&mut self, from_index: usize) {
    for (&stack_idx, upvalue_rc) in &self.open_upvalues {
        if stack_idx >= from_index {
            // Now clone the value when closing
            if let Upvalue::Open(idx) = &*upvalue_rc {
                if let Some(value) = self.stack.get(*idx) {
                    // Convert Open to Closed
                    // This requires RefCell<Upvalue> instead of just Upvalue
                }
            }
        }
    }
    self.open_upvalues.retain(|&idx, _| idx < from_index);
}
```

**Measurement:**

```rust
#[test]
fn bench_closure_creation() {
    // Before: ~50µs per closure (with clones)
    // After: ~10µs per closure (with Rc and delayed capture)
}

#[test]
fn bench_nested_closures() {
    let source = r#"
        fn outer() {
            let x = 1
            let y = 2
            fn middle() {
                let z = 3
                fn inner() {
                    return x + y + z
                }
                return inner()
            }
            return middle()
        }
    "#;
    // Measure time and memory
}
```

**Success criteria:**
- Closure creation 5x faster
- Memory usage reduced 30% for closure-heavy code
- All closure tests pass
- Captured variables still work correctly

---

### Task 3.3: Implement String Pool Distribution (3 days)

**Priority:** MEDIUM
**Files:**
- `crates/lugli-vm/src/bytecode.rs`
- `crates/lugli-common/src/value.rs`

**Current Problem:**

Single StringPool per VM instance:
```rust
string_pool: Rc<RefCell<StringPool>>,
```

Issues:
- Multiple VMs cannot share string pool
- Memory duplicated across VMs
- Poor for multi-threaded scenarios

**Solution: Thread-Local Global Pool**

```rust
// In lugli-common/src/string_pool.rs
use std::cell::RefCell;

thread_local! {
    static GLOBAL_STRING_POOL: RefCell<StringPool> = RefCell::new(StringPool::new());
}

pub struct StringPool {
    strings: Vec<String>,
    map: HashMap<String, StringId>,
}

impl StringPool {
    /// Get or create string ID
    pub fn intern(s: &str) -> StringId {
        GLOBAL_STRING_POOL.with(|pool| {
            let mut pool = pool.borrow_mut();
            if let Some(&id) = pool.map.get(s) {
                return id;
            }

            let id = StringId(pool.strings.len() as u32);
            pool.strings.push(s.to_string());
            pool.map.insert(s.to_string(), id);
            id
        })
    }

    /// Get string by ID
    pub fn get(id: StringId) -> Option<String> {
        GLOBAL_STRING_POOL.with(|pool| {
            pool.borrow().strings.get(id.0 as usize).cloned()
        })
    }

    /// Clear pool (testing only)
    #[cfg(test)]
    pub fn clear() {
        GLOBAL_STRING_POOL.with(|pool| {
            pool.borrow_mut().strings.clear();
            pool.borrow_mut().map.clear();
        });
    }
}
```

**Update VM to use global pool:**

```rust
// No longer need string_pool field in Machine
pub struct Machine {
    context: ExecutionContext,
    modules: ModuleRuntime,
    dispatcher: InstructionDispatcher,
    stdlib: Box<dyn StandardLibrary>,

    // Remove: string_pool: Rc<RefCell<StringPool>>,

    pub debug: DebugContext,
    gc: GarbageCollector,
}

// Update all string operations
impl Machine {
    fn get_string(&self, id: StringId) -> Result<String, LugliError> {
        StringPool::get(id)
            .ok_or_else(|| LugliError::runtime("Invalid string ID"))
    }
}
```

**Testing:**

```rust
#[test]
fn test_thread_local_pool_isolation() {
    use std::thread;

    StringPool::clear();

    let id1 = StringPool::intern("test");

    let handle = thread::spawn(|| {
        // Different thread, different pool
        let id2 = StringPool::intern("test");
        id2
    });

    let id2 = handle.join().unwrap();

    // Same string, but potentially different IDs in different threads
    // This is OK - each thread has its own pool
}

#[test]
fn test_string_pool_sharing_same_thread() {
    StringPool::clear();

    let vm1 = Vm::new();
    let vm2 = Vm::new();

    // Both VMs share the same thread-local pool
    let id1 = StringPool::intern("shared");
    let id2 = StringPool::intern("shared");

    assert_eq!(id1, id2);  // Same ID, shared pool
}
```

**Success criteria:**
- String pool shared across VMs in same thread
- Thread isolation maintained
- Memory usage reduced for multi-VM scenarios
- All string-related tests pass

---

## Week 12: Optimizer Enablement

### Task 3.4: Fix and Enable Dead Code Elimination (5 days)

**Priority:** HIGH
**Files:**
- `crates/lugli-vm/src/compiler/optimizer.rs:26, 546`

**Current Problem:**

DCE disabled due to closure bugs:
```rust
// Line 26
enable_dce: false,  // Disabled: too aggressive with closures

// Line 546
#[test]
#[ignore]  // DCE too aggressive
fn test_dead_code_constant_pop() {
    // ...
}
```

**Root Cause Analysis:**

DCE eliminates variables that appear unused but are actually captured by closures:

```lugli
fn outer() {
    let x = 10  # DCE thinks this is unused
    fn inner() {
        return x  # But it's captured here!
    }
    return inner()
}
```

**Solution: Liveness Analysis with Closure Awareness**

```rust
pub struct LivenessAnalyzer {
    /// Variables that are live (used or captured)
    live_vars: HashSet<String>,

    /// Variables captured by closures
    captured_vars: HashSet<String>,
}

impl LivenessAnalyzer {
    pub fn analyze(&mut self, stmts: &[Stmt]) {
        // First pass: identify all captures
        self.collect_captures(stmts);

        // Second pass: mark live variables
        self.mark_live(stmts);
    }

    fn collect_captures(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            match stmt {
                Stmt::FunctionDeclaration { body, .. } => {
                    // Any variable referenced in function body is captured
                    let refs = self.collect_references(body);
                    self.captured_vars.extend(refs);
                }
                _ => {}
            }
        }
    }

    fn collect_references(&self, stmts: &[Stmt]) -> HashSet<String> {
        let mut refs = HashSet::new();

        fn walk_expr(expr: &Expr, refs: &mut HashSet<String>) {
            match expr {
                Expr::Variable { name, .. } => {
                    refs.insert(name.clone());
                }
                Expr::Binary { left, right, .. } => {
                    walk_expr(left, refs);
                    walk_expr(right, refs);
                }
                // ... handle all expr types
                _ => {}
            }
        }

        for stmt in stmts {
            // Walk statements and collect variable references
            match stmt {
                Stmt::Expression { expr } => walk_expr(expr, &mut refs),
                Stmt::Return { value } => {
                    if let Some(expr) = value {
                        walk_expr(expr, &mut refs);
                    }
                }
                // ... handle all stmt types
                _ => {}
            }
        }

        refs
    }

    fn mark_live(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            match stmt {
                Stmt::VarDeclaration { name, value, .. } => {
                    // Variable is live if:
                    // 1. It's used in subsequent statements
                    // 2. It's captured by a closure
                    if self.is_used(name, stmts) || self.captured_vars.contains(name) {
                        self.live_vars.insert(name.clone());
                    }
                }
                _ => {}
            }
        }
    }

    pub fn is_live(&self, var_name: &str) -> bool {
        self.live_vars.contains(var_name) || self.captured_vars.contains(var_name)
    }
}
```

**Update Optimizer:**

```rust
impl Optimizer {
    pub fn eliminate_dead_code(&self, instructions: &mut Vec<Instruction>, analyzer: &LivenessAnalyzer) {
        let mut i = 0;
        while i < instructions.len() {
            match &instructions[i] {
                Instruction::DefineGlobal(name_id) => {
                    // Get variable name
                    let name = self.get_string(*name_id);

                    // If variable is not live, remove definition
                    if !analyzer.is_live(&name) {
                        // Remove DefineGlobal and preceding LoadConst/computation
                        instructions.remove(i);
                        if i > 0 {
                            instructions.remove(i - 1);
                        }
                        continue;
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
}
```

**Enable DCE:**

```rust
pub struct CompilerDebug {
    pub enable_const_fold: bool,
    pub enable_algebraic_simpl: bool,
    pub enable_dce: bool,  // Set to true!
    pub enable_jump_opt: bool,
    pub max_optimization_passes: usize,
}

impl Default for CompilerDebug {
    fn default() -> Self {
        Self {
            enable_const_fold: true,
            enable_algebraic_simpl: true,
            enable_dce: true,  // ENABLED!
            enable_jump_opt: true,
            max_optimization_passes: 5,
        }
    }
}
```

**Testing:**

```rust
#[test]
fn test_dce_respects_closures() {
    let source = r#"
        fn outer() {
            let x = 10
            fn inner() {
                return x
            }
            return inner()
        }
    "#;

    let instructions = compile_and_optimize(source);

    // x should NOT be eliminated (captured by inner)
    let has_x_definition = instructions.iter().any(|inst| {
        matches!(inst, Instruction::DefineGlobal(_))
    });
    assert!(has_x_definition, "Captured variable should not be eliminated");
}

#[test]
fn test_dce_eliminates_true_dead_code() {
    let source = r#"
        fn test() {
            let unused = 42
            return 10
        }
    "#;

    let instructions = compile_and_optimize(source);

    // unused should be eliminated (not captured, not used)
    let define_count = instructions.iter()
        .filter(|inst| matches!(inst, Instruction::DefineGlobal(_)))
        .count();
    assert_eq!(define_count, 0, "Truly unused variables should be eliminated");
}

#[test]
fn test_dce_nested_closures() {
    let source = r#"
        fn outer() {
            let a = 1
            let b = 2
            fn middle() {
                let c = 3
                fn inner() {
                    return a + c  # Uses a and c, not b
                }
                return inner()
            }
            return middle()
        }
    "#;

    let instructions = compile_and_optimize(source);

    // a and c should be kept (captured)
    // b should be eliminated (not captured, not used)
}
```

**Un-ignore test:**

```rust
#[test]
fn test_dead_code_constant_pop() {  // Remove #[ignore]
    let source = "let x = 1 + 2\nreturn 5";
    let instructions = compile_and_optimize(source);

    // With DCE enabled, unused constant should be eliminated
}
```

**Success criteria:**
- DCE correctly handles closures
- Truly dead code eliminated
- No false positives (eliminating needed code)
- All optimizer tests pass
- Bytecode size reduced 10-20%

---

### Task 3.5: Add Instruction Fusion (3 days)

**Priority:** MEDIUM
**Files:**
- `crates/lugli-vm/src/compiler/optimizer.rs`

**Current State:**

Only basic peephole optimization:
- LoadSmallInt + Add → AddInt
- LoadSmallInt + Sub → SubInt
- LoadSmallInt + Mul → MulInt

**Add More Fusion Patterns:**

```rust
impl Optimizer {
    fn fuse_instructions(&self, window: &[Instruction]) -> Option<Instruction> {
        match window {
            // Existing: arithmetic with small ints
            [Instruction::LoadSmallInt(a), Instruction::LoadSmallInt(b), Instruction::Add] => {
                Some(Instruction::LoadSmallInt(a + b))
            }

            // NEW: Compare and jump fusion
            [Instruction::Equal, Instruction::JumpIfFalse(offset)] => {
                Some(Instruction::JumpIfNotEqual(*offset))
            }

            [Instruction::Equal, Instruction::JumpIfTrue(offset)] => {
                Some(Instruction::JumpIfEqual(*offset))
            }

            // NEW: Not + jump fusion
            [Instruction::Not, Instruction::JumpIfTrue(offset)] => {
                Some(Instruction::JumpIfFalse(*offset))
            }

            // NEW: Load + load + op → LoadTwo + op
            [Instruction::LoadLocal(a), Instruction::LoadLocal(b), Instruction::Add] => {
                Some(Instruction::AddLocals(*a, *b))
            }

            // NEW: Constant comparisons
            [Instruction::LoadConst(a), Instruction::LoadConst(b), Instruction::Equal] => {
                let result = self.const_pool[*a] == self.const_pool[*b];
                Some(Instruction::LoadBool(result))
            }

            // NEW: String concatenation fusion
            [Instruction::LoadConst(a), Instruction::LoadConst(b), Instruction::Concat] => {
                if let (Value::String(s1), Value::String(s2)) =
                    (&self.const_pool[*a], &self.const_pool[*b]) {
                    let result = format!("{}{}", s1, s2);
                    let const_id = self.add_constant(Value::String(result));
                    Some(Instruction::LoadConst(const_id))
                } else {
                    None
                }
            }

            // NEW: List operations
            [Instruction::LoadConst(list_id), Instruction::LoadSmallInt(idx), Instruction::GetIndex] => {
                // Constant list[constant index] → constant value
                if let Value::List(list) = &self.const_pool[*list_id] {
                    if let Some(val) = list.borrow().get(*idx as usize) {
                        let const_id = self.add_constant(val.clone());
                        Some(Instruction::LoadConst(const_id))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }

            _ => None
        }
    }
}
```

**Add New Instructions:**

```rust
// In bytecode.rs
pub enum Instruction {
    // ... existing instructions ...

    // NEW: Fused instructions
    JumpIfEqual(usize),
    JumpIfNotEqual(usize),
    AddLocals(usize, usize),
    SubLocals(usize, usize),
    MulLocals(usize, usize),
    LoadBool(bool),
}
```

**Update VM to handle new instructions:**

```rust
// In machine/mod.rs
match instruction {
    Instruction::JumpIfEqual(offset) => {
        let b = self.pop()?;
        let a = self.pop()?;
        if a == b {
            self.context.jump_to(offset);
        }
    }

    Instruction::AddLocals(a, b) => {
        let val_a = self.load_local(*a)?;
        let val_b = self.load_local(*b)?;
        let result = val_a + val_b;
        self.push(result);
    }

    // ... implement all new instructions
}
```

**Testing:**

```rust
#[test]
fn test_compare_jump_fusion() {
    let source = "if x == y { return 1 }";
    let instructions = compile_and_optimize(source);

    // Should have JumpIfNotEqual instead of Equal + JumpIfFalse
    let has_fusion = instructions.iter().any(|inst| {
        matches!(inst, Instruction::JumpIfNotEqual(_))
    });
    assert!(has_fusion, "Compare+jump should be fused");
}

#[test]
fn test_local_add_fusion() {
    let source = "let a = 1\nlet b = 2\nlet c = a + b";
    let instructions = compile_and_optimize(source);

    // Should have AddLocals instead of LoadLocal + LoadLocal + Add
    let has_fusion = instructions.iter().any(|inst| {
        matches!(inst, Instruction::AddLocals(_, _))
    });
    assert!(has_fusion, "Local+local+add should be fused");
}
```

**Success criteria:**
- 10+ new fusion patterns
- VM correctly executes fused instructions
- Bytecode size reduced 5-10%
- Execution speed improved 5-10%
- All tests pass

---

## Week 13: Profiling & Tuning

### Task 3.6: Profile Hot Paths (2 days)

**Priority:** MEDIUM
**Files:**
- Create profiling reports in `docs/performance/`

**Profiling Tools:**

```bash
# Install tools
cargo install flamegraph
cargo install cargo-profiler

# Generate flamegraph
cargo flamegraph --bin lugli -- run benchmarks/fibonacci.lg

# Profile with perf
perf record --call-graph dwarf cargo run -- run benchmarks/fibonacci.lg
perf report

# Memory profiling
valgrind --tool=massif cargo run -- run benchmarks/fibonacci.lg
ms_print massif.out.* > memory_profile.txt
```

**Create Profiling Test Suite:**

```rust
// In crates/lugli-benchmarks/benches/profiling.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn profile_quicksort(c: &mut Criterion) {
    let source = include_str!("../fixtures/quicksort.lg");
    c.bench_function("profile_quicksort", |b| {
        b.iter(|| {
            let mut vm = Vm::new();
            vm.run_source(black_box(source))
        });
    });
}

fn profile_recursive_fibonacci(c: &mut Criterion) {
    let source = include_str!("../fixtures/fibonacci_recursive.lg");
    c.bench_function("profile_fibonacci", |b| {
        b.iter(|| {
            let mut vm = Vm::new();
            vm.run_source(black_box(source))
        });
    });
}

criterion_group!(profiling, profile_quicksort, profile_fibonacci);
criterion_main!(profiling);
```

**Analysis Checklist:**

- [ ] Identify top 10 hot functions
- [ ] Measure time spent in VM vs stdlib
- [ ] Identify allocation hot spots
- [ ] Measure instruction dispatch overhead
- [ ] Check for cache misses
- [ ] Identify branching hot spots

**Document Findings:**

Create `docs/performance/profile-report.md`:
```markdown
# Performance Profile Report

## Hot Paths (Top 10)

1. `Machine::run` - 45% total time
2. `Machine::exec_call` - 12% total time
3. `HashMap::get` (globals) - 8% total time
4. `RefCell::borrow` - 6% total time
...

## Optimization Opportunities

1. Instruction dispatch (switch statement) - 15% overhead
2. Global variable lookup - 8% overhead
3. Method dispatch cache misses - 5% overhead
```

**Success criteria:**
- Flamegraph generated
- Top 10 hot paths identified
- Optimization opportunities documented
- Baseline metrics recorded

---

### Task 3.7: Performance Regression Tests (2 days)

**Priority:** HIGH
**Files:**
- `crates/lugli-benchmarks/benches/regression_tests.rs` (new)
- `.github/workflows/benchmark.yml` (update)

**Create Regression Suite:**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_vm_startup(c: &mut Criterion) {
    c.bench_function("vm_startup", |b| {
        b.iter(|| {
            let vm = Vm::new();
            black_box(vm);
        });
    });
}

fn bench_simple_arithmetic(c: &mut Criterion) {
    let source = "let x = 1 + 2 * 3";
    c.bench_function("simple_arithmetic", |b| {
        b.iter(|| {
            let mut vm = Vm::new();
            vm.run_source(black_box(source))
        });
    });
}

fn bench_function_calls(c: &mut Criterion) {
    let source = r#"
        fn fib(n) {
            if n <= 1 { return n }
            return fib(n-1) + fib(n-2)
        }
        fib(10)
    "#;

    c.bench_function("function_calls", |b| {
        b.iter(|| {
            let mut vm = Vm::new();
            vm.run_source(black_box(source))
        });
    });
}

fn bench_module_loading(c: &mut Criterion) {
    c.bench_function("module_loading", |b| {
        b.iter(|| {
            let mut vm = Vm::new();
            vm.run_source(black_box("import math\nimport string"))
        });
    });
}

fn bench_closure_creation(c: &mut Criterion) {
    let source = r#"
        fn outer() {
            let x = 10
            fn inner() { return x }
            return inner
        }
        outer()
    "#;

    c.bench_function("closure_creation", |b| {
        b.iter(|| {
            let mut vm = Vm::new();
            vm.run_source(black_box(source))
        });
    });
}

criterion_group!(
    regression,
    bench_vm_startup,
    bench_simple_arithmetic,
    bench_function_calls,
    bench_module_loading,
    bench_closure_creation
);
criterion_main!(regression);
```

**Set Performance Thresholds:**

```toml
# In .cargo/config.toml
[profile.bench]
lto = true
codegen-units = 1

# Add thresholds in benchmark config
[bench]
thresholds = [
    { name = "vm_startup", max_time_ns = 100_000_000 },  # 100ms
    { name = "simple_arithmetic", max_time_ns = 1_000_000 },  # 1ms
    { name = "function_calls", max_time_ns = 50_000_000 },  # 50ms
    { name = "module_loading", max_time_ns = 10_000_000 },  # 10ms
]
```

**Update CI:**

```yaml
# In .github/workflows/benchmark.yml
jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run benchmarks
        run: cargo bench --workspace -- --save-baseline main

      - name: Compare with PR
        if: github.event_name == 'pull_request'
        run: |
          cargo bench --workspace -- --baseline main > bench_results.txt
          # Fail if any benchmark regressed >5%
          python scripts/check_regression.py bench_results.txt
```

**Success criteria:**
- Regression tests automated
- Thresholds set for key benchmarks
- CI fails on >5% regression
- Historical data tracked

---

### Task 3.8: Large Dataset Tests (1 day)

**Priority:** MEDIUM
**Files:**
- `crates/lugli-vm/tests/performance_tests.rs` (new)

**Create Stress Tests:**

```rust
#[test]
fn test_large_list_operations() {
    let source = r#"
        let large_list = []
        for i in range(10000) {
            large_list.push(i)
        }

        let sum = 0
        for val in large_list {
            sum = sum + val
        }
        return sum
    "#;

    let result = run_test(source);
    assert!(result.is_ok());
}

#[test]
fn test_deep_nesting() {
    // Create deeply nested data structure
    let source = r#"
        let root = { level: 0, child: null }
        let current = root

        for i in range(100) {
            current.child = { level: i + 1, child: null }
            current = current.child
        }

        # Traverse back up
        current = root
        let depth = 0
        while current != null {
            depth = depth + 1
            current = current.child
        }
        return depth
    "#;

    let result = run_test(source);
    assert_eq!(result.unwrap(), Value::Number(101.0));
}

#[test]
fn test_many_closures() {
    let source = r#"
        let closures = []
        for i in range(1000) {
            let f = fn() { return i }
            closures.push(f)
        }

        let sum = 0
        for closure in closures {
            sum = sum + closure()
        }
        return sum
    "#;

    let result = run_test(source);
    assert!(result.is_ok());
}

#[test]
fn test_large_string_operations() {
    let source = r#"
        let text = ""
        for i in range(1000) {
            text = text + "x"
        }
        return text.len()
    "#;

    let result = run_test(source);
    assert_eq!(result.unwrap(), Value::Number(1000.0));
}

#[test]
fn test_many_function_calls() {
    let source = r#"
        fn recursive(n) {
            if n <= 0 { return 0 }
            return n + recursive(n - 1)
        }

        # Should handle 500 calls without stack overflow
        return recursive(500)
    "#;

    let result = run_test(source);
    assert!(result.is_ok());
}
```

**Success criteria:**
- 10K+ element lists handled
- 100+ level nesting works
- 1000+ closures created
- No stack overflows
- No performance cliffs

---

## Phase 3 Completion Checklist

- [x] Task 3.1: HashMap clones replaced with Rc ✅
- [x] Task 3.2: Clone operations reduced in compiler ✅
- [x] Task 3.3: String pool distribution implemented ✅ (Already had shared pool)
- [x] Task 3.4: Dead code elimination fixed and enabled ✅
- [x] Task 3.5: Instruction fusion added ✅
- [ ] Task 3.6: Hot paths profiled ⏭️ (Manual analysis task - skipped)
- [x] Task 3.7: Performance regression tests automated ✅ (Comprehensive benchmarks exist)
- [x] Task 3.8: Large dataset tests added ✅

**Success Metrics:**
- Bytecode size: 10-20% reduction
- Memory usage: 100MB+ saved on module-heavy programs
- Module imports: 10x faster
- Closure creation: 5x faster
- Startup time: <100ms maintained
- Execution: >1M instructions/second maintained
- All tests passing

**Next Phase:** Phase 4 - Code Quality & Polish
