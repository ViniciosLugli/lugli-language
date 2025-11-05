# Git Hooks for Lugli Language

This directory contains git hooks that help maintain code quality.

## Setup

To enable these hooks, run:

```bash
git config core.hooksPath .githooks
```

## Available Hooks

### pre-commit

Validates code formatting before allowing commits.

**What it does:**
- Runs `cargo fmt --all -- --check`
- Prevents commits if formatting is incorrect
- Provides clear instructions on how to fix

**If the check fails:**
```bash
cargo fmt --all
git add .
git commit
```

## Why Use Git Hooks?

- ✅ Catch formatting issues before CI
- ✅ Maintain consistent code style
- ✅ Faster feedback loop
- ✅ Reduce "fix formatting" commits
