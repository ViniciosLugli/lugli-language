# Lugli Programming Language

[![Tests](https://img.shields.io/badge/tests-385%20passing-brightgreen)](https://github.com/vinicioslugli/lugli-language)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE.md)
[![Version](https://img.shields.io/badge/version-0.3.1--dev-orange)](https://github.com/vinicioslugli/lugli-language)

**Lugli** is a high-performance, dynamically-typed programming language that combines Python's simplicity with Rust's modern syntax. Built with a bytecode VM for speed and educational clarity.

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/vinicioslugli/lugli-language
cd lugli-language

# Build the interpreter
cargo build --release

# Run a program
cargo run --release -- run examples/basics/01_hello_world.lg

# Start the REPL
cargo run --release -- repl
```

### Hello World

```lugli
let name = "Lugli"
let version = "0.3.1"
print!(f"Hello from {name} v{version}!")
```

## Features

### ✅ Core Language (v0.3.1)

-   **Variables**: Mutable by default with `let`
-   **Functions**: `fn name(params) { body }` with closures
-   **Structs**: `struct Name { fields }` with methods
-   **Control Flow**: `if/elif/else`, `while`, `for`, `loop`, `break`, `continue`
-   **Pattern Matching**: `match` expressions with guards ⭐ NEW
-   **List Comprehensions**: `[x * 2 for x in list if x > 0]` ⭐ NEW
-   **Collections**: Lists `[1, 2, 3]` and dictionaries `{"key": "value"}`
-   **F-String Interpolation**: `f"Hello {name}!"`
-   **Negative Indexing**: Python-style `arr[-1]`, `str[-2]`
-   **Safe Arithmetic**: Division-by-zero protection

### 🚀 VM Architecture

-   **Bytecode Compilation**: Source → Lexer → Parser → AST → Bytecode → VM
-   **Stack-Based Execution**: >1M instructions/second
-   **Fast Startup**: <100ms cold start
-   **Low Memory**: <10MB for basic programs
-   **Native Functions**: Integrated standard library

### 📚 Standard Library

```lugli
# String methods
"hello".upper()         # "HELLO"
"hello".len()          # 5
"hello".contains("ll") # true

# List methods
let nums = [1, 2, 3]
nums.push!(4)           # [1, 2, 3, 4]
nums.pop!()             # 4
nums.len()              # 3

# Dictionary methods
let data = {"x": 10}
data.keys()             # ["x"]
data.values()           # [10]
```

## Example Programs

```lugli
# Structs with default values
struct Point {
    x: 0
    y: 0

    fn distance(self) {
        return self.x * self.x + self.y * self.y
    }
}

let p = Point { x: 3, y: 4 }
print(f"Distance: {p.distance()}")  # Distance: 25

# Loops and collections
let sum = 0
for n in [1, 2, 3, 4, 5] {
    sum = sum + n
}
print(f"Sum: {sum}")  # Sum: 15

# While with break/continue
let count = 0
while count < 10 {
    count = count + 1
    if count % 2 == 0 { continue }
    print(count)  # 1, 3, 5, 7, 9
}
```

See `examples/basics/` for more examples.

## CLI Commands

```bash
# Run a program
lugli run <file.lg>

# Start REPL
lugli repl

# Check syntax
lugli check <file.lg>

# Disassemble bytecode (debug)
lugli disassemble <file.lg>

# Show version
lugli version
```

## Architecture

Lugli is organized as a professional Rust workspace with 7 crates:

```
lugli (CLI)
    ↓
lugli-vm (Bytecode VM) ← lugli-stdlib (Native Functions)
    ↓                         ↓
lugli-parser (AST → Bytecode) ↓
    ↓                         ↓
lugli-ast (AST Definitions)   ↓
    ↓                         ↓
lugli-lexer (Source → Tokens) ↓
    ↓                         ↓
lugli-common (Shared Types) ←─┘
```

**Key Features**:

-   Zero circular dependencies
-   Clean separation of concerns
-   Comprehensive test coverage (385 tests)
-   Professional error handling with source spans

## Development

### Building

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run specific crate tests
cargo test -p lugli-vm

# Format code
cargo fmt --workspace

# Lint
cargo clippy --workspace
```

### Project Structure

```
crates/
├── lugli/          # CLI tool and REPL
├── lugli-vm/       # Bytecode virtual machine
├── lugli-stdlib/   # Standard library (native functions)
├── lugli-parser/   # Parser (tokens → AST)
├── lugli-ast/      # AST definitions
├── lugli-lexer/    # Lexer (source → tokens)
└── lugli-common/   # Shared types (Value, Error, Span)

examples/
├── basics/         # Working examples (v0.3.0)
└── future/         # Planned features (v0.4.0+)

docs/               # Documentation (coming soon)
```

## Current Status

**Version**: 0.3.1-dev (Modern Features)

### What Works

-   ✅ Complete VM-based execution pipeline
-   ✅ Core language features (variables, functions, structs, control flow)
-   ✅ **Pattern matching** with literal/identifier/wildcard patterns and guards
-   ✅ **List comprehensions** with filters
-   ✅ **Closures** with upvalue capture
-   ✅ Collections (lists, dicts) with methods
-   ✅ F-string interpolation
-   ✅ Negative indexing
-   ✅ Safe arithmetic
-   ✅ 385 tests passing (100%)

### Known Limitations

-   No nested f-strings (planned v0.4.0)
-   No module system (planned v0.4.0)
-   No type inference (planned v0.5.0)
-   No LSP server (planned v0.5.0)
-   Limited REPL (no multiline support)

See `CLAUDE.md` for complete roadmap and implementation details.

## Roadmap

### v0.3.1 (Current) - Modern Features ✅

-   ✅ List comprehensions with filters
-   ✅ Closures with upvalue capture
-   ✅ Pattern matching with guards
-   ✅ Enhanced stdlib (zip, all, any, sum, min, max, sorted, reversed)

### v0.4.0 (4-6 Weeks) - Module System & Polish

-   🔮 Module system with imports/exports
-   🔮 Enhanced error messages with stack traces
-   🔮 F-string runtime improvements
-   🔮 REPL multiline support
-   🔮 Assignment operators (`+=`, `-=`, etc.)
-   🔮 Benchmark suite

### v0.5.0 (2-3 Months) - Developer Experience

-   🔮 Type inference engine
-   🔮 LSP server (autocomplete, diagnostics)
-   🔮 VS Code extension
-   🔮 Debugger protocol
-   🔮 Package manager design

### v1.0.0 (6-12 Months) - Production Ready

-   🔮 Comprehensive standard library (50+ functions)
-   🔮 Package registry
-   🔮 Performance optimizations (JIT, constant folding)
-   🔮 Security audit
-   🔮 Production case studies

## Contributing

Contributions welcome! This is an educational project demonstrating language implementation.

**Areas for Contribution**:

-   Feature implementation (see roadmap)
-   Bug fixes
-   Documentation
-   Example programs
-   Standard library functions
-   Performance optimizations

See `CLAUDE.md` for architecture details and development guidelines.

## Testing

```bash
# Run all tests
cargo test --workspace

# Run with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test -p lugli-vm test_vm_arithmetic

# Test all examples
for file in examples/basics/*.lg; do
    cargo run -- run "$file"
done
```

**Test Statistics**:

-   Total: 385 tests
-   Passing: 385 (100%)
-   Ignored: 5 (advanced features in development)
-   Coverage: Comprehensive across all crates

## Performance

**Benchmarks** (target specifications):

-   **Startup**: <100ms ✅
-   **Execution**: >1M instructions/second ✅
-   **Memory**: <10MB for basic programs ✅
-   **Binary Size**: ~1.3MB (release build with LTO) ✅

## Documentation

-   **CLAUDE.md**: Architecture guide for AI assistants and developers
-   **examples/basics/README.md**: Guide to working examples
-   **examples/future/README.md**: Planned features roadmap

**Coming Soon**:

-   Language specification
-   API reference
-   Tutorial series
-   Design decisions document

## License

This project is licensed under the MIT License - see [LICENSE.md](LICENSE.md) for details.

**Note**: Educational project for learning language implementation.

## Credits

**Author**: Vinicios Lugli
**Inspiration**: Python (simplicity) + Rust (performance)
**Purpose**: Learning compiler/VM implementation

## Resources

-   **Repository**: https://github.com/vinicioslugli/lugli-language
-   **Issues**: https://github.com/vinicioslugli/lugli-language/issues
-   **Discussions**: https://github.com/vinicioslugli/lugli-language/discussions

---

_Built with ❤️ in Rust_
