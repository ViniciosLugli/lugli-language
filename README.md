# Lugli Programming Language

[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE.md)

A dynamic language that feels like Python, runs like Rust.

**Lugli** is a high-performance scripting language that combines Python's intuitive syntax with Rust's modern language design. Built with a bytecode VM for speed, it's perfect for rapid prototyping, learning language implementation, and scripts that need to be both readable and fast.

## Quick Look

```lugli
# Pattern matching, list comprehensions, f-strings - all working!
let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
let evens = [x * 2 for x in numbers if x % 2 == 0]

match evens.len() {
    0 => print("No evens found"),
    n => print(f"Found {n} even numbers: {evens}")
}
# Output: Found 5 even numbers: [4, 8, 12, 16, 20]
```

## What You Can Do

**Filter and transform data** with list comprehensions:
```lugli
let prices = [10, 20, 30, 40, 50]
let expensive = [p for p in prices if p > 25]
let with_tax = [p * 1.1 for p in prices]
```

**Pattern match** for cleaner control flow:
```lugli
match status {
    "success" => handle_success(),
    "error" => handle_error(),
    code if code >= 400 => handle_client_error(code),
    _ => handle_unknown()
}
```

**Build objects** with structs and methods:
```lugli
struct Counter {
    value: 0

    fn increment!(self) {
        self.value = self.value + 1
    }

    fn get(self) {
        return self.value
    }
}

let counter = Counter {}
counter.increment!()
print(counter.get())  # 1
```

**Format strings** naturally:
```lugli
let name = "Lugli"
let version = "0.4.0"
print(f"Hello from {name} v{version}!")
```

**Write closures** that capture context:
```lugli
let multiplier = 3
let triple = fn(x) { return x * multiplier }
print(triple(5))  # 15
```

## Get Started

```bash
# Clone and build
git clone https://github.com/vinicioslugli/lugli-language
cd lugli-language
cargo build --release

# Try the REPL
cargo run --release -- repl

# Run an example
cargo run --release -- run examples/basics/01_hello_world.lg
```

## Language Features

- **Variables**: `let x = 10`, mutable by default
- **Functions**: `fn add(a, b) { return a + b }`
- **Structs**: Define objects with fields and methods
- **Control Flow**: `if/elif/else`, `while`, `for`, `loop`, `match`
- **Collections**: Lists `[1, 2, 3]` and dictionaries `{"key": "value"}`
- **List Comprehensions**: `[x * 2 for x in list if x > 0]`
- **Pattern Matching**: Match with guards and wildcards
- **F-Strings**: `f"Hello {name}!"`
- **Closures**: Functions that capture their environment
- **Standard Library**: String, list, and dict methods built-in

## CLI Usage

```bash
lugli run script.lg    # Run a Lugli script
lugli repl             # Interactive mode with history
lugli check script.lg  # Check syntax without running
lugli disassemble      # Show bytecode (debug)
```

## Examples

Check out the `examples/` directory for working code:

- `examples/basics/` - Core language features
- `examples/syntax/` - Comprehensive syntax reference
- `examples/samples/` - Real programs (hangman game, etc.)

## Architecture

Lugli is built as a Rust workspace with 8 specialized crates:

- **lugli-lexer** - Tokenization
- **lugli-ast** - Abstract syntax tree
- **lugli-parser** - Source → AST
- **lugli-vm** - Bytecode compiler and VM
- **lugli-stdlib** - Native function library
- **lugli-common** - Shared types
- **lugli** - CLI and REPL
- **lugli-benchmarks** - Performance testing

The execution pipeline: Source → Lexer → Parser → AST → Bytecode → VM

## Learn More

- **CLAUDE.md** - Architecture details and internals
- **examples/** - Working code samples
- **GitHub Issues** - Questions and discussions welcome

## License

MIT License - see [LICENSE.md](LICENSE.md)

Built with Rust by [Vinicios Lugli](https://github.com/vinicioslugli)

---

*An educational project demonstrating language implementation with Python's simplicity and Rust's performance*
