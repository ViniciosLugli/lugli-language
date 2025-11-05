# Lugli Language - AI Assistant Development Guide

## 🎯 Project Objective & Language Philosophy

**Lugli** is the "child of Python and Rust" - a high-performance, high-level programming language that marries **Python's simplicity and readability** with **Rust's performance and modern syntax**.

### Language Philosophy

Lugli combines the best of both worlds:

-   **Python's** simplicity, readability, and rapid development experience
-   **Rust's** performance, safety concepts, and expressive modern syntax
-   **Dynamic typing** with optional type hints for enhanced tooling
-   **High-level abstractions** without sacrificing runtime performance

### Core Principles

1. **Readability counts** - Code should be self-documenting and obvious
2. **Performance matters** - VM-based execution with bytecode compilation for speed
3. **Developer happiness** - Intuitive syntax with excellent error messages
4. **Gradual complexity** - Simple things stay simple, complex things become possible
5. **Modern semantics** - Learn from both Python's evolution and Rust's innovations

### Design Goals

-   **High-level abstractions**: Easy syntax for complex operations
-   **Hybrid syntax**: Best patterns from both Python and Rust ecosystems
-   **High performance**: Sub-100ms startup, >1M instructions/second execution
-   **Modern tooling**: REPL, LSP, formatter, package manager
-   **Educational value**: Well-documented codebase demonstrating language implementation

---

## 🌊 Lugli Syntax Guide

> **Note**: This guide is divided into **Current Syntax** (v0.3.2, fully implemented) and **Future Syntax** (v0.4.0+, planned features).

---

## ✅ Current Syntax (v0.3.2 - Fully Implemented)

### **Variable Declaration**

```lugli
# Mutable variables
let name = "Lugli"          # ✅ Works now
mut counter = 0             # ✅ Works now
const PI = 3.14             # ✅ Works now

# With type hints (ignored at runtime) ✅
let score: f64 = 95.5
let items: List = []
```

### **Comments**

```lugli
# Single-line comment (Python-style) ✅
// Alternative single-line (Rust-style also supported) ✅
```

### **String Operations**

```lugli
# String concatenation ✅
let greeting = "Hello, " + name

# F-string interpolation ✅
let message = f"Hello, {name}! You are {age} years old."

# Format macro ✅
let formatted = format!("User: {}, ID: {}", name, id)
```

### **Print & Output**

```lugli
# Print function ✅
print("Hello, World")
print(f"Name: {name}")

# Print macro (original syntax) ✅
println!("Hello, {}", name)
```

### **Functions**

```lugli
# Basic function ✅
fn greet(name) {
    return f"Hello, {name}"
}

# Function with type hints (ignored at runtime) ✅
fn add(a: num, b: num) -> num {
    return a + b
}

# Function with closures ✅
let add = fn(a, b) { return a + b }
```

### **Structs**

```lugli
# Standard struct with methods ✅
struct Person {
    name
    age

    fn greet(self) {
        print(f"Hi, I'm {self.name}")
    }
}

# Struct with type hints (ignored at runtime) ✅
struct Point {
    x: num
    y: num = 0  # With default value
}

# Struct instantiation ✅
let person = Person { name: "Alice", age: 30 }
```

### **Control Flow**

```lugli
# If/elif/else ✅
if age >= 18 {
    print("Adult")
} elif age >= 13 {
    print("Teen")
} else {
    print("Child")
}

# Pattern matching ✅
match status {
    "success" => handle_success(),
    "error" => handle_error(),
    code if code >= 400 => handle_client_error(code),
    _ => handle_unknown()
}
```

### **Collections**

```lugli
# Lists ✅
let numbers = [1, 2, 3, 4, 5]

# List comprehensions ✅
let squares = [x * x for x in numbers]
let evens = [x for x in numbers if x % 2 == 0]

# Dictionaries ✅
let person = {
    "name": "Alice",
    "age": 30,
    "active": true
}
```

### **Loops & Iteration**

```lugli
# For loops ✅
for item in collection {
    print(item)
}

for (index, item) in enumerate(collection) {
    print(f"{index}: {item}")
}

# While loops ✅
while condition {
    # ...
}

# Infinite loops ✅
loop {
    if should_break { break }
    if should_skip { continue }
}
```

### **Import System (Basic)**

```lugli
# Dynamic imports ✅
import!("./module.lg")
```

### **Method Naming Conventions**

```lugli
# Parser supports ! and ? suffixes for user-defined methods ✅
struct Counter {
    value

    fn increment!(self) {  # Mutating method
        self.value = self.value + 1
    }

    fn is_zero?(self) {  # Query method
        return self.value == 0
    }
}

let counter = Counter { value: 0 }
counter.increment!()
let check = counter.is_zero?()

# Stdlib methods use plain names (no suffix convention)
list.push(item)  # ✅ Works
text.upper()     # ✅ Works
```

---

## 🔮 Future Syntax (v0.4.0+ - Planned)

### **Advanced Features** ❌ NOT YET IMPLEMENTED

```lugli
# Multiple assignment
let x, y = 10, 20

# Destructuring
let {name, age} = person

# Spread operator
let [first, ...rest] = numbers

# Impl blocks
impl Person {
    fn new(name, age) {
        return Person { name, age }
    }
}

# Generic types
struct Container<T> {
    value: T
}

# Inheritance
struct Student : Person {
    grade: num
}

# Result/Option types
fn divide(a, b) -> Result<num, str> {
    if b == 0 {
        return Err("Division by zero")
    }
    return Ok(a / b)
}
```

### **File-Based Imports** ⚠️ PARTIALLY IMPLEMENTED

```lugli
# Python-style imports (parsed but not fully functional)
import math
from stdlib import List, Dict
from math import pi, sqrt
```

### **Advanced I/O** ❌ NOT YET IMPLEMENTED

```lugli
# File operations
let file = open("data.txt", "r")
let content = file.read()
file.close()

# HTTP requests
let response = http.get("https://api.example.com")

# Process spawning
let result = process.run("ls", ["-la"])
```

---

## 🏗️ Architecture Overview

The project is organized as a **Rust workspace** with **7 professional crates** in the `crates/` directory:

### Crate Dependencies (Bottom to Top)

```
lugli (CLI)
    ↓
lugli-vm (Bytecode Virtual Machine) ← lugli-stdlib (Standard Library)
    ↓                                      ↓
lugli-parser (AST → Bytecode Compiler)     ↓
    ↓                                      ↓
lugli-ast (AST Definitions)                ↓
    ↓                                      ↓
lugli-lexer (Source → Tokens)              ↓
    ↓                                      ↓
    └────── lugli-common (Shared Types) ──┘
```

### Core Components

#### `lugli-lexer`

-   **Purpose**: Tokenization of source code
-   **Technology**: `logos` crate for fast lexical analysis
-   **Output**: Stream of tokens with location information
-   **Key files**: `token.rs`, `scanner.rs`

#### `lugli-ast`

-   **Purpose**: Abstract Syntax Tree definitions
-   **Features**: Immutable nodes, visitor pattern, span tracking
-   **Key files**: `expr.rs`, `stmt.rs`, `visitor.rs`

#### `lugli-parser`

-   **Purpose**: Parse tokens into AST
-   **Technology**: Recursive descent with Pratt parsing for operators
-   **Features**: Error recovery, detailed error messages
-   **Key files**: `parser.rs`, `precedence.rs`, `error.rs`

#### `lugli-common`

-   **Purpose**: Shared types and foundational components
-   **Features**: Value enum, error types, source spans, common utilities
-   **Key files**: `value.rs`, `error.rs`, `span.rs`
-   **Used by**: All other crates for type consistency

#### `lugli-vm`

-   **Purpose**: High-performance bytecode virtual machine
-   **Architecture**: Stack-based execution with integrated stdlib functions
-   **Features**: AST compilation, native function registry, error recovery
-   **Key files**: `machine.rs`, `compiler.rs`, `bytecode.rs`
-   **Performance**: >1M instructions/second, sub-100ms startup

#### `lugli-stdlib`

-   **Purpose**: Native function implementations for VM
-   **Architecture**: Function registry system with VM integration
-   **Method naming**: `!` for mutating, `?` for queries
-   **Key modules**: `core/`, `io/`, `time/`
-   **Integration**: Direct VM function calls via registry

#### `lugli` (main)

-   **Purpose**: CLI tool and REPL
-   **Technology**: `clap` for CLI, custom REPL with syntax highlighting
-   **Commands**: `run`, `repl`, `check`, `fmt`, `test`

---

## 🔧 Code Patterns & Conventions

### Error Handling

```rust
// Use Result for fallible operations
fn parse_expression(&mut self) -> Result<Expr, ParseError> {
    // Implementation
}

// Use custom error types with thiserror
#[derive(Error, Debug)]
pub enum LexError {
    #[error("Unexpected character: {0}")]
    UnexpectedChar(char),
    #[error("Unterminated string literal")]
    UnterminatedString,
}
```

### Memory Management

```rust
// Use Rc<RefCell<>> for shared mutable state
type Environment = Rc<RefCell<HashMap<String, Value>>>;

// Prefer &str over String for parameters
fn lookup_variable(&self, name: &str) -> Option<&Value>

// Use Box for recursive types
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>
    },
}
```

### Documentation Policy

-   **MINIMAL DOCUMENTATION**: Only add comments/docs when absolutely necessary
-   No redundant rustdoc comments that just repeat the function name
-   Only document complex algorithms, non-obvious behavior, or public APIs that need explanation
-   Prefer self-documenting code with clear naming over comments

### Migration Policy

-   **REMOVE OLD/DEPRECATED CODE**: Always remove outdated code during migration
-   Don't just copy-paste old code - modernize and improve it
-   Remove TODO comments that are no longer relevant
-   Clean up unused imports and dependencies
-   Update to latest API patterns and best practices

### Type Definitions

```rust
// Always implement Debug and Display
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: Span,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}({})", self.kind, self.lexeme)
    }
}
```

### Testing Patterns

```rust
// Unit tests in each crate
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_number() {
        let mut lexer = Lexer::new("42");
        assert_eq!(lexer.next_token(), Token::number(42.0));
    }
}

// Property-based testing for complex behavior
use proptest::prelude::*;

proptest! {
    #[test]
    fn parse_roundtrip(source in ".*") {
        let tokens = lexer::tokenize(&source)?;
        let ast = parser::parse(tokens)?;
        let regenerated = ast.to_string();
        // Verify semantic equivalence
    }
}
```

### Standard Library Conventions

```rust
// Method naming: ! for mutating, ? for queries
impl StringObject {
    pub fn upper(&self) -> String { ... }        // Returns new value
    pub fn append(&mut self, s: &str) { ... }    // Mutates in place
    pub fn len(&self) -> usize { ... }          // Query method
}

// Use in Lugli:
// name.upper()     // Non-mutating
// name.append!("!") // Mutating
// name.len?()      // Query
```

---

## 🧪 Testing Strategy

### Test Organization

```
crates/
├── lugli-lexer/
│   ├── tests/
│   │   ├── lexer_tests.rs      # Unit tests
│   │   └── fixtures/           # Test input files
│   └── benches/
│       └── lexer_bench.rs      # Performance tests
└── ...

tests/                          # Integration tests
├── fixtures/                   # .lg test programs
├── language/                   # Feature tests
└── integration/                # End-to-end tests
```

### Test Categories

#### Unit Tests (per crate)

-   **Lexer**: All token types, error cases, Unicode
-   **Parser**: All language constructs, error recovery
-   **Runtime**: Variable scoping, function calls, stdlib functions
-   **Stdlib**: Every method, edge cases, error conditions

#### Integration Tests (workspace level)

-   **Language features**: Control flow, functions, structs
-   **Standard library**: Cross-module interactions
-   **Error handling**: Parse errors, runtime errors
-   **Performance**: Regression tests, memory usage

#### Benchmarks

```rust
// Use criterion for benchmarking
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn lexer_benchmark(c: &mut Criterion) {
    let source = include_str!("../fixtures/large_program.lg");
    c.bench_function("tokenize large program", |b| {
        b.iter(|| {
            let lexer = Lexer::new(black_box(source));
            lexer.collect::<Vec<_>>()
        })
    });
}
```

### Performance Targets

-   **Lexer**: >1MB/s tokenization speed
-   **Parser**: >500KB/s parsing speed
-   **VM**: >1M instructions/second
-   **Startup**: <100ms cold start
-   **Memory**: <10MB for basic programs

---

## 🔄 Syntax Implementation Guide

### **Priority Implementation Order**

#### Phase 2.1: Core Syntax Updates

1. **Lexer Updates**:

    - Add `let` and `mut` keywords
    - Support both `#` and `//` comments
    - Add `match` keyword for pattern matching
    - Add f-string tokenization (`f"..."`)

2. **Parser Updates**:

    - Replace `create` with `let`/`mut` in variable declarations
    - Update struct methods to use `self` instead of `this`
    - Add pattern matching syntax parsing
    - Support optional type hints syntax

3. **Example Updates**:
    - Convert all examples to new syntax
    - Add new examples for modern features
    - Update REPL prompts and error messages

#### Phase 2.2: Modern Features

1. **String Interpolation**: F-string support
2. **List Comprehensions**: Python-style syntax
3. **Pattern Matching**: Rust-inspired match expressions
4. **Type Hints**: Optional typing for better tooling

#### Phase 2.3: Standard Library

1. **Pythonic APIs**: `print()`, `len()`, `str()`, `int()`
2. **Enumerate Function**: For indexed iteration
3. **Range Function**: For numeric iteration
4. **Modern Collections**: Enhanced dict and list APIs

### **Syntax Migration Strategy**

#### Backward Compatibility

-   Keep `create` as alias for `let` during transition
-   Support both comment styles (`#` and `//`)
-   Maintain `println!` alongside new `print()`
-   Keep current method naming (`!` and `?`) working

#### Migration Path

1. **Phase 1**: Add new syntax alongside old
2. **Phase 2**: Update all examples to new style
3. **Phase 3**: Deprecate old syntax with warnings
4. **Phase 4**: Remove deprecated syntax

### **Testing Each Syntax Feature**

For each new syntax element:

1. **Lexer tests**: Tokenization correctness
2. **Parser tests**: AST generation
3. **Runtime tests**: Execution behavior
4. **Example files**: Usage demonstration
5. **Error tests**: Helpful error messages
6. **Performance tests**: No regressions

### **Example File Organization**

```
examples/
├── syntax/
│   ├── 01_variables.lg       # let, mut, const
│   ├── 02_functions.lg       # fn with type hints
│   ├── 03_structs.lg         # struct with self
│   ├── 04_control_flow.lg    # if/elif/else, match
│   ├── 05_collections.lg     # lists, dicts, comprehensions
│   ├── 06_strings.lg         # f-strings, formatting
│   ├── 07_loops.lg           # for/while/loop
│   ├── 08_imports.lg         # import system
│   ├── 09_comments.lg        # comment styles
│   └── 10_types.lg           # type hints
├── tutorials/
│   ├── hello_world.lg        # First program
│   ├── calculator.lg         # Basic arithmetic
│   └── todo_list.lg          # Data structures
└── samples/
    ├── hangman.lg            # Game implementation
    ├── web_server.lg         # Async example
    └── data_analysis.lg      # List comprehensions
```

---

## 🚀 Development Workflow

### Adding New Language Features

1. **Design Phase**

    - Document syntax in `docs/language_spec.md`
    - Add examples to `examples/syntax/`
    - Consider backward compatibility

2. **Implementation Phase**

    - Update lexer if new tokens needed
    - Add AST nodes in `lugli-ast`
    - Update parser for new syntax
    - Implement runtime behavior
    - Add stdlib support if applicable

3. **Testing Phase**

    - Unit tests for each component
    - Integration tests for feature
    - Performance benchmarks
    - Update documentation

4. **Review Phase**
    - Run full test suite
    - Check performance regressions
    - Update examples
    - Code review

### Common Development Tasks

#### Build & Test

```bash
# Build all crates
cargo build --workspace

# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p lugli-lexer

# Run benchmarks
cargo bench

# Check for issues
cargo clippy --workspace
cargo fmt --workspace
```

#### Running Programs

```bash
# Run a Lugli program
cargo run -- run examples/hello_world.lg

# Start REPL
cargo run -- repl

# Check syntax without running
cargo run -- check examples/syntax/functions.lg

# Format Lugli code
cargo run -- fmt examples/
```

#### Debugging

```bash
# Debug mode with extra logging
RUST_LOG=debug cargo run -- run program.lg

# Print AST for debugging
cargo run -- parse --ast program.lg

# Print bytecode
cargo run -- compile --bytecode program.lg
```

---

## 🔍 Debugging & Troubleshooting

### Common Issues

#### Parser Errors

-   Check token precedence in `precedence.rs`
-   Verify error recovery synchronization points
-   Test with minimal failing cases

#### Runtime Errors

-   Use `RUST_LOG=debug` for detailed execution tracing
-   Check variable scoping in environment
-   Verify function call argument matching

#### Performance Issues

-   Profile with `cargo bench`
-   Use `perf` or `flamegraph` for detailed analysis
-   Check for excessive allocations

### Development Environment

```bash
# Install development tools
cargo install cargo-watch
cargo install cargo-expand
cargo install flamegraph

# Watch and rebuild on changes
cargo watch -x "test --workspace"

# Expand macros for debugging
cargo expand -p lugli-lexer

# Generate flame graphs
cargo flamegraph --bin lugli -- run large_program.lg
```

---

## 📚 Key Dependencies

### Core Dependencies

-   **logos** (0.15.1): Fast lexical analysis
-   **clap** (4.5.48): Command-line interface
-   **thiserror** (2.0.16): Error handling
-   **chrono** (0.4.42): Date/time operations

### Development Dependencies

-   **criterion**: Performance benchmarking
-   **proptest**: Property-based testing
-   **insta**: Snapshot testing for parser output
-   **pretty_assertions**: Better test failure output

### Optional Features

-   **serde**: Serialization for AST caching
-   **rayon**: Parallel compilation
-   **mimalloc**: Fast memory allocator

---

## 🎓 Learning Resources

### Understanding the Codebase

1. Start with `lugli-common` - foundational types
2. Read `lugli-lexer` - simplest component
3. Study `lugli-ast` for data structures
4. Explore `lugli-parser` for algorithm complexity
5. Examine `lugli-vm` for execution model
6. Review `lugli-stdlib` for language features

### Recommended Reading

-   "Crafting Interpreters" by Robert Nystrom
-   "Language Implementation Patterns" by Terence Parr
-   Rust Programming Language Book (for Rust patterns)

### Contributing Guidelines

-   All code must have tests
-   Performance regressions require justification
-   Breaking changes need migration guide
-   Documentation must be updated

---

## 🔄 Development Status

### ✅ v0.3.2 - Production-Safe VM

-   **Tests**: 385 passing, 5 ignored (390 total)
-   **VM**: Stack-based bytecode execution, panic-safe
-   **Features**: F-strings, list comprehensions, pattern matching, closures, structs
-   **Safety**: All RefCell panics fixed, bounds checks, proper error handling

### ✅ Complete Features

-   7-crate workspace with clean architecture
-   VM-only execution (no tree-walking interpreter)
-   Pattern matching with guards
-   List comprehensions with filters
-   F-string interpolation
-   Closures with upvalue capture
-   Struct methods and properties
-   Enhanced error messages

### 📋 Next (see [Roadmap](docs/development/IMPLEMENTATION_ROADMAP.md))

-   Module system (file-based imports)
-   REPL enhancements (rustyline, multiline)
-   Performance optimizations

---

**Version**: 0.3.2
**Last Updated**: 2025-10-28

---

_This guide is for AI assistants working on the Lugli language project. Keep it updated as the project evolves._
