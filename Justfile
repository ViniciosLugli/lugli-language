# Justfile for Lugli Language Development
# Run `just` to see all available commands

# Default recipe shows help
default:
    @just --list

# Build all crates in workspace
build:
    cargo build --workspace

# Build in release mode
build-release:
    cargo build --workspace --release

# Run all tests
test:
    cargo test --workspace

# Run tests for a specific crate
test-crate crate:
    cargo test -p {{crate}}

# Run tests with output
test-verbose:
    cargo test --workspace -- --nocapture

# Run a specific test by name
test-name name:
    cargo test --workspace {{name}} -- --nocapture

# Format all code
fmt:
    cargo fmt --all

# Check formatting without modifying files
fmt-check:
    cargo fmt --all -- --check

# Run clippy linter
lint:
    cargo clippy --workspace

# Run clippy with auto-fix suggestions
lint-fix:
    cargo clippy --workspace --fix --allow-dirty

# Check code without building
check:
    cargo check --workspace

# Run a Lugli program
run file:
    cargo run -- run {{file}}

# Start the REPL
repl:
    cargo run -- repl

# Run example by name (e.g. `just example basics/01_hello_world.lg`)
example name:
    cargo run -- run examples/{{name}}

# Run all examples in basics/
examples-basics:
    #!/usr/bin/env bash
    for file in examples/basics/*.lg; do
        echo "Running $file..."
        cargo run -- run "$file" || echo "Failed: $file"
    done

# Run all examples in samples/
examples-samples:
    #!/usr/bin/env bash
    for file in examples/samples/*.lg; do
        echo "Running $file..."
        timeout 5s cargo run -- run "$file" || echo "Failed/Timeout: $file"
    done

# Clean build artifacts
clean:
    cargo clean

# Full verification (fmt, lint, test)
verify: fmt lint test
    @echo "✅ All checks passed!"

# Watch and rebuild on changes
watch:
    cargo watch -x "test --workspace"

# Watch and run specific test
watch-test name:
    cargo watch -x "test {{name}} -- --nocapture"

# Count lines of code
loc:
    @echo "=== Lugli Language Code Statistics ==="
    @tokei crates/ -e crates/*/tests

# Show workspace dependency tree
deps:
    cargo tree --workspace

# Update dependencies
update:
    cargo update

# Run benchmarks
bench:
    cargo bench

# Run benchmarks for specific crate
bench-crate crate:
    cargo bench -p {{crate}}

# Generate documentation
doc:
    cargo doc --workspace --no-deps

# Open documentation in browser
doc-open:
    cargo doc --workspace --no-deps --open

# Check for outdated dependencies
outdated:
    cargo outdated

# Run security audit
audit:
    cargo audit

# Install Lugli CLI locally
install:
    cargo install --path crates/lugli

# Uninstall Lugli CLI
uninstall:
    cargo uninstall lugli

# Full cleanup and rebuild
rebuild: clean build

# Quick check before commit
pre-commit: fmt lint test-verbose
    @echo "✅ Ready to commit!"
