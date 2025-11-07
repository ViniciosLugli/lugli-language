# Phase 2: Architecture Refactoring (4-5 weeks)

**Goal:** Decompose god objects, establish proper abstractions

**Priority:** P0 - Required for scalability

**Dependencies:** Phase 1 must be complete

**Success Criteria:**
- ✅ Machine reduced from 15 to 5 responsibilities
- ✅ Clean stdlib abstraction layer
- ✅ Single canonical API
- ✅ 50% reduction in coupling metrics
- ✅ All tests passing

---

## Week 5-6: Machine Decomposition

### Task 2.1: Extract ExecutionContext (5 days)

**Priority:** CRITICAL
**Files:**
- `crates/lugli-vm/src/machine/context.rs` (new)
- `crates/lugli-vm/src/machine/mod.rs` (refactor)
- All machine/*.rs submodules

**Current Problem:**

Machine struct has 15+ responsibilities:
```rust
pub struct Machine {
    pub stack: Vec<Value>,              // Stack management
    pub globals: HashMap<String, Value>, // Global variables
    pub ip: usize,                      // Instruction pointer
    call_stack: Vec<CallFrame>,         // Call stack management
    pub debug: DebugContext,            // Debug info
    module_cache: ModuleCache,          // Module system
    // ... 10 more fields
}
```

**Solution: Extract Execution Context**

**Create `crates/lugli-vm/src/machine/context.rs`:**

```rust
use crate::value::Value;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

/// Execution context holds runtime state for VM execution
pub struct ExecutionContext {
    /// The value stack
    stack: Vec<Value>,

    /// Global variables
    globals: HashMap<String, Value>,

    /// Call stack frames
    call_stack: Vec<CallFrame>,

    /// Open upvalues for closures (stack_index -> upvalue)
    open_upvalues: HashMap<usize, Rc<RefCell<Value>>>,

    /// Current instruction pointer
    ip: usize,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            globals: HashMap::new(),
            call_stack: Vec::new(),
            open_upvalues: HashMap::new(),
            ip: 0,
        }
    }

    // Stack operations
    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    pub fn peek(&self, distance: usize) -> Option<&Value> {
        if distance >= self.stack.len() {
            return None;
        }
        self.stack.get(self.stack.len() - 1 - distance)
    }

    pub fn stack_len(&self) -> usize {
        self.stack.len()
    }

    pub fn truncate_stack(&mut self, len: usize) {
        self.stack.truncate(len);
    }

    // Global variable operations
    pub fn define_global(&mut self, name: String, value: Value) {
        self.globals.insert(name, value);
    }

    pub fn get_global(&self, name: &str) -> Option<&Value> {
        self.globals.get(name)
    }

    pub fn set_global(&mut self, name: &str, value: Value) -> Result<(), String> {
        if !self.globals.contains_key(name) {
            return Err(format!("Undefined variable '{}'", name));
        }
        self.globals.insert(name.to_string(), value);
        Ok(())
    }

    // Call stack operations
    pub fn push_frame(&mut self, frame: CallFrame) {
        self.call_stack.push(frame);
    }

    pub fn pop_frame(&mut self) -> Option<CallFrame> {
        self.call_stack.pop()
    }

    pub fn current_frame(&self) -> Option<&CallFrame> {
        self.call_stack.last()
    }

    pub fn current_frame_mut(&mut self) -> Option<&mut CallFrame> {
        self.call_stack.last_mut()
    }

    pub fn call_depth(&self) -> usize {
        self.call_stack.len()
    }

    // Upvalue operations
    pub fn capture_upvalue(&mut self, stack_index: usize) -> Result<Rc<RefCell<Value>>, String> {
        if let Some(existing) = self.open_upvalues.get(&stack_index) {
            return Ok(Rc::clone(existing));
        }

        let value = self.stack.get(stack_index)
            .ok_or("Stack index out of bounds")?
            .clone();
        let upvalue = Rc::new(RefCell::new(value));
        self.open_upvalues.insert(stack_index, Rc::clone(&upvalue));
        Ok(upvalue)
    }

    pub fn close_upvalues(&mut self, from_index: usize) {
        self.open_upvalues.retain(|&idx, _| idx < from_index);
    }

    // Instruction pointer operations
    pub fn advance_ip(&mut self, offset: usize) {
        self.ip += offset;
    }

    pub fn jump_to(&mut self, target: usize) {
        self.ip = target;
    }

    pub fn current_ip(&self) -> usize {
        self.ip
    }

    pub fn reset_ip(&mut self) {
        self.ip = 0;
    }
}

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub bytecode_id: usize,
    pub return_ip: usize,
    pub stack_base: usize,
}
```

**Update `crates/lugli-vm/src/machine/mod.rs`:**

```rust
use crate::machine::context::ExecutionContext;

pub struct Machine {
    // Replace individual fields with context
    context: ExecutionContext,

    // Keep non-execution state
    pub debug: DebugContext,
    module_cache: ModuleCache,
    module_resolver: ModuleResolver,
    bytecode_registry: HashMap<usize, Rc<Bytecode>>,
    module_globals: HashMap<usize, Rc<HashMap<String, Value>>>,
    next_bytecode_id: usize,
    string_pool: Rc<RefCell<StringPool>>,
    method_registry: MethodRegistry,
    method_cache: HashMap<(usize, usize), u32>,
    gc: GarbageCollector,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            context: ExecutionContext::new(),
            // ... rest of initialization
        }
    }

    // Delegate to context
    fn push(&mut self, value: Value) {
        self.context.push(value);
    }

    fn pop(&mut self) -> Result<Value, LugliError> {
        self.context.pop()
            .ok_or_else(|| LugliError::runtime("Stack underflow"))
    }

    fn peek(&self, distance: usize) -> Result<&Value, LugliError> {
        self.context.peek(distance)
            .ok_or_else(|| LugliError::runtime("Stack underflow"))
    }

    // ... delegate all context operations
}
```

**Update all machine/*.rs files:**

Each file needs to use `self.context.*` instead of direct field access:

- `machine/stack_ops.rs` → use `self.context.push()`, `self.context.pop()`
- `machine/variable_ops.rs` → use `self.context.get_global()`, `self.context.set_global()`
- `machine/control_flow.rs` → use `self.context.push_frame()`, `self.context.pop_frame()`
- `machine/object_ops.rs` → use `self.context.capture_upvalue()`
- All others → update accordingly

**Testing Strategy:**

1. Add context unit tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_stack_operations() {
        let mut ctx = ExecutionContext::new();
        ctx.push(Value::Number(42.0));
        assert_eq!(ctx.stack_len(), 1);
        let val = ctx.pop().unwrap();
        assert_eq!(val, Value::Number(42.0));
    }

    #[test]
    fn test_context_globals() {
        let mut ctx = ExecutionContext::new();
        ctx.define_global("x".to_string(), Value::Number(10.0));
        assert_eq!(ctx.get_global("x"), Some(&Value::Number(10.0)));
    }
}
```

2. Ensure all existing VM tests still pass

**Success criteria:**
- ExecutionContext fully encapsulates runtime state
- Machine delegates to context
- All tests pass
- No performance regression

---

### Task 2.2: Extract ModuleRuntime (3 days)

**Priority:** HIGH
**Files:**
- `crates/lugli-vm/src/machine/module_runtime.rs` (new)
- `crates/lugli-vm/src/machine/mod.rs` (refactor)
- `crates/lugli-vm/src/module.rs` (update)

**Create `crates/lugli-vm/src/machine/module_runtime.rs`:**

```rust
use std::collections::HashMap;
use std::rc::Rc;
use std::path::{Path, PathBuf};
use crate::bytecode::Bytecode;
use crate::module::{ModuleCache, ModuleResolver};
use crate::value::Value;
use crate::error::LugliError;

/// Manages module loading, caching, and global state
pub struct ModuleRuntime {
    /// Cache of loaded modules (path → bytecode_id)
    module_cache: ModuleCache,

    /// Resolves module paths
    module_resolver: ModuleResolver,

    /// Registry of bytecode by ID
    bytecode_registry: HashMap<usize, Rc<Bytecode>>,

    /// Module-specific globals
    module_globals: HashMap<usize, Rc<HashMap<String, Value>>>,

    /// Next bytecode ID
    next_bytecode_id: usize,

    /// Current file being executed
    current_file: Option<PathBuf>,
}

impl ModuleRuntime {
    pub fn new() -> Self {
        Self {
            module_cache: ModuleCache::new(),
            module_resolver: ModuleResolver::new(),
            bytecode_registry: HashMap::new(),
            module_globals: HashMap::new(),
            next_bytecode_id: 0,
            current_file: None,
        }
    }

    pub fn register_bytecode(&mut self, bytecode: Bytecode) -> usize {
        let id = self.next_bytecode_id;
        self.next_bytecode_id += 1;
        self.bytecode_registry.insert(id, Rc::new(bytecode));
        id
    }

    pub fn get_bytecode(&self, id: usize) -> Option<&Rc<Bytecode>> {
        self.bytecode_registry.get(&id)
    }

    pub fn load_module(&mut self, module_path: &str, base_path: Option<&Path>)
        -> Result<usize, LugliError> {
        // Check cache first
        if let Some(&bytecode_id) = self.module_cache.get(module_path) {
            return Ok(bytecode_id);
        }

        // Resolve path
        let resolved_path = self.module_resolver.resolve(module_path, base_path)?;

        // Load and compile module
        let source = std::fs::read_to_string(&resolved_path)
            .map_err(|e| LugliError::runtime(format!("Failed to read module: {}", e)))?;

        // This needs to trigger compilation - delegate to caller
        // Return resolved path and let caller compile
        Err(LugliError::runtime("Module compilation delegated to caller"))
    }

    pub fn cache_module(&mut self, path: String, bytecode_id: usize) {
        self.module_cache.insert(path, bytecode_id);
    }

    pub fn save_module_globals(&mut self, bytecode_id: usize, globals: HashMap<String, Value>) {
        self.module_globals.insert(bytecode_id, Rc::new(globals));
    }

    pub fn get_module_globals(&self, bytecode_id: usize) -> Option<&Rc<HashMap<String, Value>>> {
        self.module_globals.get(&bytecode_id)
    }

    pub fn set_current_file(&mut self, path: Option<PathBuf>) {
        self.current_file = path;
    }

    pub fn current_file(&self) -> Option<&Path> {
        self.current_file.as_deref()
    }
}
```

**Update Machine to use ModuleRuntime:**

```rust
pub struct Machine {
    context: ExecutionContext,
    modules: ModuleRuntime,  // Replaces 5 module-related fields

    // Remaining fields
    pub debug: DebugContext,
    string_pool: Rc<RefCell<StringPool>>,
    method_registry: MethodRegistry,
    method_cache: HashMap<(usize, usize), u32>,
    gc: GarbageCollector,
}
```

**Success criteria:**
- ModuleRuntime encapsulates all module state
- Machine uses modules field
- Module loading logic cleaner
- All module tests pass

---

### Task 2.3: Create InstructionDispatcher (3 days)

**Priority:** HIGH
**Files:**
- `crates/lugli-vm/src/machine/dispatcher.rs` (new)
- `crates/lugli-vm/src/machine/mod.rs` (refactor)

**Create `crates/lugli-vm/src/machine/dispatcher.rs`:**

```rust
use std::collections::HashMap;
use lugli_stdlib::MethodRegistry;
use crate::value::Value;
use crate::error::LugliError;
use lugli_common::StringPool;

/// Handles method dispatch and caching
pub struct InstructionDispatcher {
    /// Registry of methods
    method_registry: MethodRegistry,

    /// Method call cache (type_hash, method_hash) → method_id
    method_cache: HashMap<(usize, usize), u32>,
}

impl InstructionDispatcher {
    pub fn new() -> Self {
        let mut method_registry = MethodRegistry::new();
        method_registry.register_all();

        Self {
            method_registry,
            method_cache: HashMap::new(),
        }
    }

    pub fn dispatch_method(
        &mut self,
        type_name: &str,
        method_name: &str,
        args: &[Value],
        pool: &mut StringPool,
    ) -> Result<Value, LugliError> {
        // Try cache first
        let type_hash = Self::hash_string(type_name);
        let method_hash = Self::hash_string(method_name);
        let cache_key = (type_hash, method_hash);

        if let Some(&method_id) = self.method_cache.get(&cache_key) {
            return self.method_registry.call_by_id(method_id, args, pool);
        }

        // Cache miss - lookup and cache
        let result = self.method_registry.call(type_name, method_name, args, pool)?;

        if let Some(method_id) = self.method_registry.get_method_id(type_name, method_name) {
            self.method_cache.insert(cache_key, method_id);
        }

        Ok(result)
    }

    fn hash_string(s: &str) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish() as usize
    }

    pub fn clear_cache(&mut self) {
        self.method_cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.method_cache.len()
    }
}
```

**Update Machine:**

```rust
pub struct Machine {
    context: ExecutionContext,
    modules: ModuleRuntime,
    dispatcher: InstructionDispatcher,  // Replaces method_registry + method_cache

    pub debug: DebugContext,
    string_pool: Rc<RefCell<StringPool>>,
    gc: GarbageCollector,
}
```

**Success criteria:**
- InstructionDispatcher handles all method calls
- Method caching preserved
- Performance no worse than before
- All method call tests pass

---

## Week 7-8: Stdlib Abstraction

### Task 2.4: Create StandardLibrary Trait (4 days)

**Priority:** CRITICAL
**Files:**
- `crates/lugli-common/src/stdlib.rs` (new)
- `crates/lugli-stdlib/src/lib.rs` (implement trait)
- `crates/lugli-vm/src/machine/mod.rs` (use trait)

**Create `crates/lugli-common/src/stdlib.rs`:**

```rust
use crate::value::Value;
use crate::error::LugliError;
use std::collections::HashMap;

/// Function signature for native functions
pub type NativeFunction = fn(&[Value], &mut StringPool) -> Result<Value, LugliError>;

/// Trait for standard library implementations
pub trait StandardLibrary: Send + Sync {
    /// Get global functions (print, len, etc.)
    fn get_global_functions(&self) -> HashMap<String, NativeFunction>;

    /// Get method registry for types
    fn get_method_registry(&self) -> Box<dyn MethodRegistry>;

    /// Get library name and version
    fn info(&self) -> (&'static str, &'static str);
}

/// Trait for method registries
pub trait MethodRegistry: Send + Sync {
    /// Call a method on a value
    fn call(
        &self,
        type_name: &str,
        method_name: &str,
        args: &[Value],
        pool: &mut StringPool,
    ) -> Result<Value, LugliError>;

    /// Check if method exists
    fn has_method(&self, type_name: &str, method_name: &str) -> bool;

    /// Get all methods for a type
    fn methods_for_type(&self, type_name: &str) -> Vec<&'static str>;
}

/// Factory for creating standard library instances
pub trait StandardLibraryFactory {
    fn create() -> Box<dyn StandardLibrary>;
}
```

**Update `crates/lugli-stdlib/src/lib.rs`:**

```rust
use lugli_common::stdlib::*;

pub struct LugliStdlib;

impl StandardLibrary for LugliStdlib {
    fn get_global_functions(&self) -> HashMap<String, NativeFunction> {
        let mut functions = HashMap::new();
        functions.insert("print".to_string(), io::print as NativeFunction);
        functions.insert("len".to_string(), core::length as NativeFunction);
        // ... register all functions
        functions
    }

    fn get_method_registry(&self) -> Box<dyn MethodRegistry> {
        Box::new(methods::MethodRegistry::new())
    }

    fn info(&self) -> (&'static str, &'static str) {
        ("lugli-stdlib", env!("CARGO_PKG_VERSION"))
    }
}

impl StandardLibraryFactory for LugliStdlib {
    fn create() -> Box<dyn StandardLibrary> {
        Box::new(LugliStdlib)
    }
}

// Adapter for existing MethodRegistry
impl lugli_common::stdlib::MethodRegistry for methods::MethodRegistry {
    fn call(
        &self,
        type_name: &str,
        method_name: &str,
        args: &[Value],
        pool: &mut StringPool,
    ) -> Result<Value, LugliError> {
        self.call(type_name, method_name, args, pool)
    }

    fn has_method(&self, type_name: &str, method_name: &str) -> bool {
        self.get_method_id(type_name, method_name).is_some()
    }

    fn methods_for_type(&self, type_name: &str) -> Vec<&'static str> {
        self.list_methods(type_name)
    }
}
```

**Update Machine to use trait:**

```rust
pub struct Machine {
    context: ExecutionContext,
    modules: ModuleRuntime,
    dispatcher: InstructionDispatcher,
    stdlib: Box<dyn StandardLibrary>,  // Trait instead of concrete type

    pub debug: DebugContext,
    string_pool: Rc<RefCell<StringPool>>,
    gc: GarbageCollector,
}

impl Machine {
    pub fn new() -> Self {
        Self::with_stdlib(LugliStdlib::create())
    }

    pub fn with_stdlib(stdlib: Box<dyn StandardLibrary>) -> Self {
        let global_functions = stdlib.get_global_functions();
        let method_registry = stdlib.get_method_registry();

        Self {
            context: ExecutionContext::new(),
            modules: ModuleRuntime::new(),
            dispatcher: InstructionDispatcher::with_registry(method_registry),
            stdlib,
            // ... rest of initialization
        }
    }
}
```

**Testing:**

```rust
// Create minimal test stdlib
struct TestStdlib;

impl StandardLibrary for TestStdlib {
    fn get_global_functions(&self) -> HashMap<String, NativeFunction> {
        let mut fns = HashMap::new();
        fns.insert("test_fn".to_string(), |args, _| Ok(Value::Number(42.0)));
        fns
    }

    fn get_method_registry(&self) -> Box<dyn MethodRegistry> {
        Box::new(EmptyMethodRegistry)
    }

    fn info(&self) -> (&'static str, &'static str) {
        ("test", "0.1.0")
    }
}

#[test]
fn test_custom_stdlib() {
    let mut machine = Machine::with_stdlib(Box::new(TestStdlib));
    // Test that custom stdlib works
}
```

**Success criteria:**
- StandardLibrary trait fully functional
- Existing stdlib implements trait
- Machine uses trait abstraction
- Can swap stdlib implementations
- All tests pass

---

### Task 2.5: Improve Stdlib Error Context (2 days)

**Priority:** MEDIUM
**Files:**
- All `crates/lugli-stdlib/src/**/*.rs` files
- `crates/lugli-common/src/error.rs` (add context)

**Current Problem:**

```rust
pub fn list_length(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "list.len")?;
    match &args[0] {
        Value::List(l) => Ok(Value::Number(l.borrow().len() as f64)),
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}
```

No source location, no helpful suggestions.

**Solution:**

Update function signature:
```rust
pub type NativeFunction = fn(&[Value], &mut StringPool, Option<&SourceContext>)
    -> Result<Value, LugliError>;

#[derive(Debug, Clone)]
pub struct SourceContext {
    pub file: Option<String>,
    pub line: usize,
    pub column: usize,
}
```

Update error creation:
```rust
pub fn list_length(
    args: &[Value],
    _pool: &mut StringPool,
    ctx: Option<&SourceContext>
) -> Result<Value, LugliError> {
    check_arity(args, 1, "list.len", ctx)?;
    match &args[0] {
        Value::List(l) => Ok(Value::Number(l.borrow().len() as f64)),
        _ => {
            let mut err = LugliError::type_error("list", args[0].type_name());
            if let Some(context) = ctx {
                err = err.with_context(context.clone());
            }
            Err(err)
        }
    }
}
```

**Update all stdlib files - systematic approach:**

1. Update core/list.rs
2. Update core/string.rs
3. Update core/dict.rs
4. Update io/print.rs
5. Update all others

**Success criteria:**
- All stdlib functions accept SourceContext
- Error messages include file/line when available
- Helpful error suggestions added
- All tests pass with better errors

---

### Task 2.6: Implement Method Provider Pattern (2 days)

**Priority:** MEDIUM
**Files:**
- `crates/lugli-stdlib/src/methods.rs` (refactor)
- `crates/lugli-stdlib/src/core/*.rs` (update)

**Create trait-based registration:**

```rust
pub trait MethodProvider {
    fn type_name() -> &'static str;
    fn register_methods(registry: &mut MethodRegistry);
}

// In core/string.rs
pub struct StringMethods;

impl MethodProvider for StringMethods {
    fn type_name() -> &'static str {
        "string"
    }

    fn register_methods(registry: &mut MethodRegistry) {
        registry.register("string", "len", string_length);
        registry.register("string", "upper", string_upper);
        registry.register("string", "lower", string_lower);
        // ... all string methods
    }
}

// In methods.rs
impl MethodRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            methods: HashMap::new(),
            hash_methods: HashMap::new(),
            hash_names: HashMap::new(),
        };

        // Auto-register all providers
        StringMethods::register_methods(&mut registry);
        ListMethods::register_methods(&mut registry);
        DictMethods::register_methods(&mut registry);

        registry
    }
}
```

**Success criteria:**
- Trait-based registration working
- Easy to add new method providers
- All existing methods registered
- All tests pass

---

## Week 9: API Consolidation

### Task 2.7: Implement VM Builder Pattern (3 days)

**Priority:** HIGH
**Files:**
- `crates/lugli-vm/src/lib.rs` (major refactor)
- `crates/lugli-vm/src/builder.rs` (new)
- `crates/lugli/src/main.rs` (update to use builder)

**Current Problem:**

6 different entry points:
```rust
pub fn compile(...)
pub fn compile_with_source(...)
pub fn run(bytecode: &Bytecode) -> Result<Value, VmError>
pub fn compile_and_run(...)
pub fn compile_and_run_with_source(...)
pub fn run_with_vm(vm: &mut Machine, bytecode: &Bytecode)
```

**Create `crates/lugli-vm/src/builder.rs`:**

```rust
use crate::Machine;
use crate::bytecode::Bytecode;
use crate::value::Value;
use crate::error::VmError;
use lugli_common::StringPool;
use std::rc::Rc;
use std::cell::RefCell;

pub struct VmBuilder {
    source_file: Option<String>,
    source_code: Option<String>,
    string_pool: Option<Rc<RefCell<StringPool>>>,
    stdlib: Option<Box<dyn StandardLibrary>>,
    debug_mode: bool,
}

impl VmBuilder {
    pub fn new() -> Self {
        Self {
            source_file: None,
            source_code: None,
            string_pool: None,
            stdlib: None,
            debug_mode: false,
        }
    }

    pub fn with_source(mut self, file: String, code: String) -> Self {
        self.source_file = Some(file);
        self.source_code = Some(code);
        self
    }

    pub fn with_shared_pool(mut self, pool: Rc<RefCell<StringPool>>) -> Self {
        self.string_pool = Some(pool);
        self
    }

    pub fn with_stdlib(mut self, stdlib: Box<dyn StandardLibrary>) -> Self {
        self.stdlib = Some(stdlib);
        self
    }

    pub fn debug(mut self, enabled: bool) -> Self {
        self.debug_mode = enabled;
        self
    }

    pub fn build(self) -> Machine {
        let mut machine = if let Some(stdlib) = self.stdlib {
            Machine::with_stdlib(stdlib)
        } else {
            Machine::new()
        };

        if let Some(pool) = self.string_pool {
            machine.set_string_pool(pool);
        }

        if self.debug_mode {
            machine.enable_debug();
        }

        if let Some(file) = self.source_file {
            machine.set_source_file(file);
        }

        machine
    }
}

// Fluent API for common operations
pub struct Vm {
    machine: Machine,
}

impl Vm {
    pub fn builder() -> VmBuilder {
        VmBuilder::new()
    }

    pub fn new() -> Self {
        Self {
            machine: Machine::new(),
        }
    }

    pub fn compile(&mut self, program: &Program, span_map: SpanMap) -> Result<Bytecode, VmError> {
        // Compile AST to bytecode
        let compiler = Compiler::new_with_pool(self.machine.string_pool());
        compiler.compile(program, span_map)
            .map_err(VmError::from)
    }

    pub fn run(&mut self, bytecode: &Bytecode) -> Result<Value, VmError> {
        self.machine.run(bytecode)
            .map_err(VmError::from)
    }

    pub fn compile_and_run(&mut self, program: &Program, span_map: SpanMap)
        -> Result<Value, VmError> {
        let bytecode = self.compile(program, span_map)?;
        self.run(&bytecode)
    }
}
```

**Update public API in lib.rs:**

```rust
// Simple API
pub use builder::{Vm, VmBuilder};

// For advanced users
pub use machine::Machine;
pub use bytecode::Bytecode;

// Recommended usage:
// let mut vm = Vm::new();
// let bytecode = vm.compile(&program, span_map)?;
// let result = vm.run(&bytecode)?;

// Or one-shot:
// let mut vm = Vm::new();
// let result = vm.compile_and_run(&program, span_map)?;

// Or with custom config:
// let vm = Vm::builder()
//     .with_stdlib(custom_stdlib)
//     .debug(true)
//     .build();
```

**Update CLI to use new API:**

```rust
// In src/main.rs
fn run_file(path: &Path) -> Result<(), Box<dyn Error>> {
    let source = fs::read_to_string(path)?;
    let tokens = tokenize(&source)?;
    let (program, span_map) = parse(&tokens)?;

    let mut vm = Vm::builder()
        .with_source(path.display().to_string(), source)
        .debug(std::env::var("LUGLI_DEBUG").is_ok())
        .build();

    let result = vm.compile_and_run(&program, span_map)?;
    println!("{}", result);
    Ok(())
}
```

**Success criteria:**
- Builder pattern fully functional
- All 6 old entry points replaced
- CLI uses new API
- Migration guide written
- All tests updated
- All examples updated

---

### Task 2.8: Update All Documentation (2 days)

**Priority:** HIGH
**Files:**
- `CLAUDE.md` (update architecture section)
- `docs/architecture.md` (new)
- `docs/migration-v0.5.md` (new migration guide)
- All crate README.md files

**Update CLAUDE.md:**

```markdown
## Architecture Overview (v0.5.0)

The project is organized as a Rust workspace with 8 professional crates:

### Updated Architecture

```
lugli (CLI) → all crates
    ↓
lugli-vm → ExecutionContext + ModuleRuntime + InstructionDispatcher
    ↓
lugli-stdlib → implements StandardLibrary trait
    ↓
lugli-common (Foundation + Traits)
```

### Key Changes from v0.4.0

1. **Machine Decomposition**: Machine struct reduced from 15 to 5 core responsibilities
2. **Stdlib Abstraction**: StandardLibrary trait allows swappable implementations
3. **Unified API**: Single VmBuilder pattern replaces 6 entry points
4. **Better Encapsulation**: ExecutionContext, ModuleRuntime, InstructionDispatcher

### Core Components

#### ExecutionContext
- Encapsulates: stack, globals, call_stack, upvalues, IP
- Location: `crates/lugli-vm/src/machine/context.rs`
- Purpose: Runtime execution state

#### ModuleRuntime
- Encapsulates: module_cache, module_resolver, bytecode_registry
- Location: `crates/lugli-vm/src/machine/module_runtime.rs`
- Purpose: Module loading and caching

#### InstructionDispatcher
- Encapsulates: method_registry, method_cache
- Location: `crates/lugli-vm/src/machine/dispatcher.rs`
- Purpose: Method dispatch with caching

#### StandardLibrary Trait
- Location: `crates/lugli-common/src/stdlib.rs`
- Purpose: Pluggable stdlib implementations
- Implementations: LugliStdlib (default)
```

**Create `docs/migration-v0.5.md`:**

```markdown
# Migration Guide: v0.4.0 → v0.5.0

## Breaking Changes

### VM API Consolidation

**Old (v0.4.0):**
```rust
use lugli_vm::{compile_and_run, Machine};

let result = compile_and_run(&program, span_map)?;
```

**New (v0.5.0):**
```rust
use lugli_vm::Vm;

let mut vm = Vm::new();
let result = vm.compile_and_run(&program, span_map)?;
```

### Custom Stdlib

**Old:** Direct dependency on lugli_stdlib

**New:**
```rust
use lugli_vm::{Vm, VmBuilder};
use lugli_common::StandardLibrary;

struct CustomStdlib;
impl StandardLibrary for CustomStdlib {
    // implement trait
}

let vm = Vm::builder()
    .with_stdlib(Box::new(CustomStdlib))
    .build();
```

### Direct Machine Access

**Old:**
```rust
let mut machine = Machine::new();
machine.stack.push(value);  // Direct access
```

**New:**
```rust
let mut vm = Vm::new();
// Use methods instead of direct field access
```

## Deprecation Warnings

The following functions are deprecated and will be removed in v0.6.0:
- `compile()`
- `compile_with_source()`
- `run_with_vm()`

Use `Vm` API instead.
```

**Create architecture diagrams:**

```markdown
# Architecture Documentation

## Component Diagram

```
┌─────────────────────────────────────────┐
│              Lugli CLI                  │
└───────────────┬─────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────┐
│             Vm (Facade)                 │
│  ┌────────────────────────────────┐    │
│  │         Machine                │    │
│  │  ┌──────────────────────────┐  │    │
│  │  │  ExecutionContext        │  │    │
│  │  │  - stack                 │  │    │
│  │  │  - globals               │  │    │
│  │  │  - call_stack            │  │    │
│  │  └──────────────────────────┘  │    │
│  │  ┌──────────────────────────┐  │    │
│  │  │  ModuleRuntime           │  │    │
│  │  │  - module_cache          │  │    │
│  │  │  - bytecode_registry     │  │    │
│  │  └──────────────────────────┘  │    │
│  │  ┌──────────────────────────┐  │    │
│  │  │  InstructionDispatcher   │  │    │
│  │  │  - method_registry       │  │    │
│  │  │  - method_cache          │  │    │
│  │  └──────────────────────────┘  │    │
│  └────────────────────────────────┘    │
└───────────────┬─────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────┐
│     StandardLibrary (trait)             │
│  ┌──────────────────────────────────┐   │
│  │       LugliStdlib (impl)         │   │
│  └──────────────────────────────────┘   │
└─────────────────────────────────────────┘
```
```

**Success criteria:**
- CLAUDE.md fully updated
- Architecture docs complete
- Migration guide comprehensive
- All examples updated
- README files current

---

## Phase 2 Completion Checklist

- [ ] Task 2.1: ExecutionContext extracted
- [ ] Task 2.2: ModuleRuntime extracted
- [ ] Task 2.3: InstructionDispatcher created
- [ ] Task 2.4: StandardLibrary trait implemented
- [ ] Task 2.5: Stdlib error context improved
- [ ] Task 2.6: Method provider pattern implemented
- [ ] Task 2.7: VM builder pattern implemented
- [ ] Task 2.8: All documentation updated

**Success Metrics:**
- Machine responsibilities: 15 → 5
- Coupling metrics: 50% reduction
- Public API: 6 entry points → 1 unified API
- Test coverage maintained at 80%+
- All tests passing
- Zero performance regression

**Next Phase:** Phase 3 - Performance Optimization
