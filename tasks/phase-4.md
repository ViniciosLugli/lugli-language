# Phase 4: Code Quality & Polish (2-3 weeks)

**Goal:** Clean up tech debt, improve maintainability

**Priority:** P2 - Quality improvements

**Dependencies:** Phases 1-3 complete

**Success Criteria:**
- ✅ All files <500 lines
- ✅ Comprehensive documentation
- ✅ Clear architectural patterns
- ✅ Production-ready for v1.0
- ✅ Zero technical debt

---

## Week 14-15: Module Organization

### Task 4.1: Split Expression Compiler (4 days)

**Priority:** HIGH
**Files:**
- `crates/lugli-vm/src/compiler/expressions.rs` (1,017 lines → split into 6 modules)
- `crates/lugli-vm/src/compiler/expressions/mod.rs` (new)
- `crates/lugli-vm/src/compiler/expressions/*.rs` (new)

**Current Problem:**

Single 1,017-line file with all expression compilation logic:
- Binary operations (~50 lines)
- Function calls (~100 lines)
- List/dict construction (~80 lines)
- F-string interpolation (~200+ lines)
- Comprehensions (~150+ lines)
- Match expressions (~150+ lines)
- Method calls (~100+ lines)

**Solution: Split by Expression Type**

**Create directory structure:**
```
crates/lugli-vm/src/compiler/expressions/
├── mod.rs              # Main entry point + simple expressions
├── binary.rs           # Binary operations
├── calls.rs            # Function and method calls
├── collections.rs      # List and dict literals
├── fstrings.rs         # F-string interpolation
├── comprehensions.rs   # List comprehensions
└── patterns.rs         # Match expressions
```

**Create `expressions/mod.rs`:**

```rust
mod binary;
mod calls;
mod collections;
mod fstrings;
mod comprehensions;
mod patterns;

use crate::compiler::Compiler;
use lugli_ast::Expr;
use crate::error::CompileError;

impl Compiler {
    /// Main expression compilation entry point
    pub fn compile_expr(&mut self, expr: &Expr) -> Result<(), CompileError> {
        match expr {
            // Simple expressions handled here
            Expr::Literal { value } => self.compile_literal(value),
            Expr::Variable { name } => self.compile_variable(name),
            Expr::Grouping { expr } => self.compile_expr(expr),

            // Delegate complex expressions to submodules
            Expr::Binary { left, operator, right } => {
                binary::compile_binary(self, left, operator, right)
            }

            Expr::Call { callee, arguments } => {
                calls::compile_call(self, callee, arguments)
            }

            Expr::MethodCall { object, method, arguments } => {
                calls::compile_method_call(self, object, method, arguments)
            }

            Expr::List { elements } => {
                collections::compile_list(self, elements)
            }

            Expr::Dict { pairs } => {
                collections::compile_dict(self, pairs)
            }

            Expr::FString { parts } => {
                fstrings::compile_fstring(self, parts)
            }

            Expr::ListComprehension { element, iterator, condition } => {
                comprehensions::compile_list_comprehension(self, element, iterator, condition)
            }

            Expr::Match { value, arms } => {
                patterns::compile_match(self, value, arms)
            }

            _ => Err(CompileError::Unsupported("Unknown expression type")),
        }
    }

    fn compile_literal(&mut self, value: &Token) -> Result<(), CompileError> {
        // Inline for simple cases
        match &value.kind {
            TokenKind::Number(n) => self.emit_load_number(*n),
            TokenKind::String(s) => self.emit_load_string(s),
            TokenKind::True => self.emit_constant(Value::Bool(true)),
            TokenKind::False => self.emit_constant(Value::Bool(false)),
            TokenKind::Null => self.emit_constant(Value::Null),
            _ => Err(CompileError::InvalidLiteral),
        }
    }

    fn compile_variable(&mut self, name: &str) -> Result<(), CompileError> {
        if let Some(local_idx) = self.resolve_local(name) {
            self.emit(Instruction::LoadLocal(local_idx));
        } else if let Some(upvalue_idx) = self.resolve_upvalue(name) {
            self.emit(Instruction::LoadUpvalue(upvalue_idx));
        } else {
            let name_id = self.intern_string(name);
            self.emit(Instruction::LoadGlobal(name_id));
        }
        Ok(())
    }
}
```

**Create `expressions/binary.rs`:**

```rust
use crate::compiler::Compiler;
use crate::error::CompileError;
use lugli_ast::{Expr, TokenKind};
use crate::bytecode::Instruction;

pub fn compile_binary(
    compiler: &mut Compiler,
    left: &Expr,
    operator: &TokenKind,
    right: &Expr,
) -> Result<(), CompileError> {
    // Compile left operand
    compiler.compile_expr(left)?;

    // Compile right operand
    compiler.compile_expr(right)?;

    // Emit operation
    match operator {
        TokenKind::Plus => compiler.emit(Instruction::Add),
        TokenKind::Minus => compiler.emit(Instruction::Subtract),
        TokenKind::Star => compiler.emit(Instruction::Multiply),
        TokenKind::Slash => compiler.emit(Instruction::Divide),
        TokenKind::Percent => compiler.emit(Instruction::Modulo),
        TokenKind::StarStar => compiler.emit(Instruction::Power),

        TokenKind::EqualEqual => compiler.emit(Instruction::Equal),
        TokenKind::BangEqual => {
            compiler.emit(Instruction::Equal);
            compiler.emit(Instruction::Not);
        }
        TokenKind::Greater => compiler.emit(Instruction::Greater),
        TokenKind::GreaterEqual => {
            compiler.emit(Instruction::Less);
            compiler.emit(Instruction::Not);
        }
        TokenKind::Less => compiler.emit(Instruction::Less),
        TokenKind::LessEqual => {
            compiler.emit(Instruction::Greater);
            compiler.emit(Instruction::Not);
        }

        TokenKind::And => compile_and(compiler, left, right)?,
        TokenKind::Or => compile_or(compiler, left, right)?,

        _ => return Err(CompileError::InvalidOperator),
    }

    Ok(())
}

fn compile_and(
    compiler: &mut Compiler,
    left: &Expr,
    right: &Expr,
) -> Result<(), CompileError> {
    // Short-circuit evaluation for 'and'
    compiler.compile_expr(left)?;
    let jump_to_end = compiler.emit_jump_if_false_placeholder();
    compiler.emit(Instruction::Pop);
    compiler.compile_expr(right)?;
    compiler.patch_jump(jump_to_end);
    Ok(())
}

fn compile_or(
    compiler: &mut Compiler,
    left: &Expr,
    right: &Expr,
) -> Result<(), CompileError> {
    // Short-circuit evaluation for 'or'
    compiler.compile_expr(left)?;
    let jump_to_end = compiler.emit_jump_if_true_placeholder();
    compiler.emit(Instruction::Pop);
    compiler.compile_expr(right)?;
    compiler.patch_jump(jump_to_end);
    Ok(())
}
```

**Create `expressions/calls.rs`:**

```rust
use crate::compiler::Compiler;
use crate::error::CompileError;
use lugli_ast::Expr;

pub fn compile_call(
    compiler: &mut Compiler,
    callee: &Expr,
    arguments: &[Expr],
) -> Result<(), CompileError> {
    // Compile callee
    compiler.compile_expr(callee)?;

    // Compile arguments
    for arg in arguments {
        compiler.compile_expr(arg)?;
    }

    // Emit call instruction
    compiler.emit(Instruction::Call {
        arg_count: arguments.len(),
    });

    Ok(())
}

pub fn compile_method_call(
    compiler: &mut Compiler,
    object: &Expr,
    method: &str,
    arguments: &[Expr],
) -> Result<(), CompileError> {
    // Compile object
    compiler.compile_expr(object)?;

    // Compile arguments
    for arg in arguments {
        compiler.compile_expr(arg)?;
    }

    // Intern method name
    let method_id = compiler.intern_string(method);

    // Emit method call instruction
    compiler.emit(Instruction::CallMethod {
        name_id: method_id,
        arg_count: arguments.len(),
    });

    Ok(())
}
```

**Create `expressions/collections.rs`:**

```rust
pub fn compile_list(
    compiler: &mut Compiler,
    elements: &[Expr],
) -> Result<(), CompileError> {
    // Compile each element
    for elem in elements {
        compiler.compile_expr(elem)?;
    }

    // Create list
    compiler.emit(Instruction::BuildList(elements.len()));

    Ok(())
}

pub fn compile_dict(
    compiler: &mut Compiler,
    pairs: &[(String, Expr)],
) -> Result<(), CompileError> {
    // Compile each key-value pair
    for (key, value) in pairs {
        let key_id = compiler.intern_string(key);
        compiler.emit(Instruction::LoadConst(key_id));
        compiler.compile_expr(value)?;
    }

    // Create dict
    compiler.emit(Instruction::BuildDict(pairs.len()));

    Ok(())
}
```

**Create `expressions/fstrings.rs` (extract 200+ lines):**

```rust
use crate::compiler::Compiler;
use crate::error::CompileError;
use lugli_ast::{FStringPart, Expr};

pub fn compile_fstring(
    compiler: &mut Compiler,
    parts: &[FStringPart],
) -> Result<(), CompileError> {
    if parts.is_empty() {
        compiler.emit_load_string("");
        return Ok(());
    }

    // Compile each part
    for (i, part) in parts.iter().enumerate() {
        match part {
            FStringPart::Literal(s) => {
                compiler.emit_load_string(s)?;
            }
            FStringPart::Expression(expr) => {
                compile_fstring_expression(compiler, expr)?;
            }
        }

        // Concatenate with previous parts
        if i > 0 {
            compiler.emit(Instruction::Concat);
        }
    }

    Ok(())
}

fn compile_fstring_expression(
    compiler: &mut Compiler,
    expr: &Expr,
) -> Result<(), CompileError> {
    // Compile expression
    compiler.compile_expr(expr)?;

    // Convert to string if needed
    compiler.emit(Instruction::ToString);

    Ok(())
}

// Helper for nested f-strings
pub fn compile_nested_fstring(
    compiler: &mut Compiler,
    parts: &[FStringPart],
    depth: usize,
) -> Result<(), CompileError> {
    const MAX_FSTRING_DEPTH: usize = 3;

    if depth > MAX_FSTRING_DEPTH {
        return Err(CompileError::TooDeep("F-string nesting exceeds maximum"));
    }

    compile_fstring(compiler, parts)
}
```

**Create `expressions/comprehensions.rs`:**

```rust
pub fn compile_list_comprehension(
    compiler: &mut Compiler,
    element: &Expr,
    iterator: &Expr,
    condition: &Option<Expr>,
) -> Result<(), CompileError> {
    // Create empty list
    compiler.emit(Instruction::BuildList(0));

    // Compile iterator
    compiler.compile_expr(iterator)?;

    // Start loop
    let loop_start = compiler.current_position();

    // Get next value or break
    compiler.emit(Instruction::IterNext);
    let loop_end = compiler.emit_jump_if_null_placeholder();

    // Optional condition
    if let Some(cond) = condition {
        compiler.compile_expr(cond)?;
        let skip_append = compiler.emit_jump_if_false_placeholder();

        // Compile element and append to list
        compiler.compile_expr(element)?;
        compiler.emit(Instruction::ListAppend);

        compiler.patch_jump(skip_append);
    } else {
        // No condition - always append
        compiler.compile_expr(element)?;
        compiler.emit(Instruction::ListAppend);
    }

    // Jump back to loop start
    compiler.emit_jump_backward(loop_start);

    // Patch loop end
    compiler.patch_jump(loop_end);

    Ok(())
}
```

**Create `expressions/patterns.rs`:**

```rust
pub fn compile_match(
    compiler: &mut Compiler,
    value: &Expr,
    arms: &[MatchArm],
) -> Result<(), CompileError> {
    // Compile value to match
    compiler.compile_expr(value)?;

    let mut arm_ends = Vec::new();

    for arm in arms {
        // Duplicate value for comparison
        compiler.emit(Instruction::Dup);

        // Compile pattern
        compile_pattern(compiler, &arm.pattern)?;

        // Check if guard condition exists
        if let Some(guard) = &arm.guard {
            compiler.emit(Instruction::Equal);
            let skip_arm = compiler.emit_jump_if_false_placeholder();

            compiler.compile_expr(guard)?;
            let skip_arm2 = compiler.emit_jump_if_false_placeholder();

            // Pattern and guard matched - compile body
            compiler.emit(Instruction::Pop); // Pop value
            compiler.compile_expr(&arm.body)?;

            // Jump to end after executing arm
            let jump_end = compiler.emit_jump_placeholder();
            arm_ends.push(jump_end);

            compiler.patch_jump(skip_arm);
            compiler.patch_jump(skip_arm2);
        } else {
            // No guard - just check pattern
            compiler.emit(Instruction::Equal);
            let skip_arm = compiler.emit_jump_if_false_placeholder();

            // Pattern matched - compile body
            compiler.emit(Instruction::Pop); // Pop value
            compiler.compile_expr(&arm.body)?;

            // Jump to end
            let jump_end = compiler.emit_jump_placeholder();
            arm_ends.push(jump_end);

            compiler.patch_jump(skip_arm);
        }
    }

    // No pattern matched - error
    compiler.emit(Instruction::MatchError);

    // Patch all arm endings
    for jump in arm_ends {
        compiler.patch_jump(jump);
    }

    Ok(())
}

fn compile_pattern(
    compiler: &mut Compiler,
    pattern: &Pattern,
) -> Result<(), CompileError> {
    match pattern {
        Pattern::Literal(value) => {
            compiler.compile_literal(value)?;
        }
        Pattern::Variable(name) => {
            // Bind variable
            let name_id = compiler.intern_string(name);
            compiler.emit(Instruction::BindPattern(name_id));
        }
        Pattern::Wildcard => {
            // Always matches
            compiler.emit(Instruction::LoadBool(true));
        }
    }
    Ok(())
}
```

**Success criteria:**
- expressions.rs split into 7 files (<200 lines each)
- All expression compilation working
- No duplicate code
- All compiler tests pass

---

### Task 4.2: Reorganize Parser Helpers (3 days)

**Priority:** MEDIUM
**Files:**
- `crates/lugli-parser/src/helpers.rs` (732 lines → split into 4 modules)
- `crates/lugli-parser/src/helpers/mod.rs` (new)

**Current Problem:**

Single 732-line file with mixed responsibilities:
- Pattern parsing (~150 lines)
- Type hint parsing (~80 lines)
- Parameter parsing (~50 lines)
- Argument parsing (~60 lines)
- Block parsing (~40 lines)
- Utility methods (~350 lines)

**Solution: Split by Responsibility**

**Create directory structure:**
```
crates/lugli-parser/src/helpers/
├── mod.rs          # Re-exports
├── patterns.rs     # Pattern matching syntax
├── types.rs        # Type hints and annotations
├── parameters.rs   # Function parameters
└── arguments.rs    # Function arguments
```

**Create `helpers/mod.rs`:**

```rust
mod patterns;
mod types;
mod parameters;
mod arguments;

pub use patterns::*;
pub use types::*;
pub use parameters::*;
pub use arguments::*;

// Keep simple utility methods here
impl Parser {
    pub fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    pub fn advance(&mut self) -> Option<Token> {
        if !self.is_at_end() {
            let token = self.tokens[self.current].clone();
            self.current += 1;
            Some(token)
        } else {
            None
        }
    }

    pub fn check(&self, kind: TokenKind) -> bool {
        self.peek().map_or(false, |t| t.kind == kind)
    }

    pub fn consume(&mut self, kind: TokenKind, message: &str) -> Result<Token, ParseError> {
        if self.check(kind) {
            Ok(self.advance().unwrap())
        } else {
            Err(ParseError::Expected {
                expected: format!("{:?}", kind),
                found: self.peek().map(|t| t.lexeme.clone()).unwrap_or_default(),
                message: message.to_string(),
            })
        }
    }
}
```

**Create `helpers/patterns.rs`:**

```rust
use crate::parser::Parser;
use crate::error::ParseError;
use lugli_ast::{Pattern, Expr};
use lugli_lexer::TokenKind;

impl Parser {
    pub fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        if self.check(TokenKind::Underscore) {
            self.advance();
            return Ok(Pattern::Wildcard);
        }

        if self.check(TokenKind::Identifier) {
            let name = self.advance().unwrap().lexeme;
            return Ok(Pattern::Variable(name));
        }

        // Literal patterns
        if let Some(literal) = self.try_parse_literal()? {
            return Ok(Pattern::Literal(literal));
        }

        Err(ParseError::InvalidPattern)
    }

    pub fn parse_match_arms(&mut self) -> Result<Vec<MatchArm>, ParseError> {
        let mut arms = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            let arm = self.parse_match_arm()?;
            arms.push(arm);

            if !self.check(TokenKind::Comma) {
                break;
            }
            self.advance(); // Consume comma
        }

        Ok(arms)
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm, ParseError> {
        let pattern = self.parse_pattern()?;

        // Optional guard
        let guard = if self.check(TokenKind::If) {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        self.consume(TokenKind::FatArrow, "Expected '=>' in match arm")?;

        let body = Box::new(self.parse_expr()?);

        Ok(MatchArm { pattern, guard, body })
    }
}
```

**Create `helpers/types.rs`:**

```rust
impl Parser {
    pub fn parse_type_hint(&mut self) -> Result<Option<TypeHint>, ParseError> {
        if !self.check(TokenKind::Colon) {
            return Ok(None);
        }

        self.advance(); // Consume ':'

        let type_name = self.consume(TokenKind::Identifier, "Expected type name")?;

        // Optional generic parameters
        let generics = if self.check(TokenKind::Less) {
            self.parse_generic_parameters()?
        } else {
            vec![]
        };

        Ok(Some(TypeHint {
            name: type_name.lexeme,
            generics,
        }))
    }

    fn parse_generic_parameters(&mut self) -> Result<Vec<TypeHint>, ParseError> {
        self.consume(TokenKind::Less, "Expected '<'")?;

        let mut params = Vec::new();

        loop {
            if self.check(TokenKind::Greater) {
                break;
            }

            let param_type = self.consume(TokenKind::Identifier, "Expected type name")?;
            params.push(TypeHint {
                name: param_type.lexeme,
                generics: vec![],
            });

            if !self.check(TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        self.consume(TokenKind::Greater, "Expected '>'")?;

        Ok(params)
    }

    pub fn parse_return_type(&mut self) -> Result<Option<TypeHint>, ParseError> {
        if !self.check(TokenKind::Arrow) {
            return Ok(None);
        }

        self.advance(); // Consume '->'
        self.parse_type_hint()
    }
}
```

**Create `helpers/parameters.rs`:**

```rust
impl Parser {
    pub fn parse_parameters(&mut self) -> Result<Vec<Parameter>, ParseError> {
        self.consume(TokenKind::LeftParen, "Expected '(' after function name")?;

        let mut parameters = Vec::new();

        if !self.check(TokenKind::RightParen) {
            loop {
                let param = self.parse_parameter()?;
                parameters.push(param);

                if !self.check(TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;

        Ok(parameters)
    }

    fn parse_parameter(&mut self) -> Result<Parameter, ParseError> {
        let name = self.consume(TokenKind::Identifier, "Expected parameter name")?;

        // Optional type hint
        let type_hint = self.parse_type_hint()?;

        // Optional default value
        let default = if self.check(TokenKind::Equal) {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        Ok(Parameter {
            name: name.lexeme,
            type_hint,
            default,
        })
    }

    pub fn parse_struct_fields(&mut self) -> Result<Vec<StructField>, ParseError> {
        let mut fields = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            let field = self.parse_struct_field()?;
            fields.push(field);

            // Fields can be separated by commas or newlines
            if self.check(TokenKind::Comma) || self.check(TokenKind::Newline) {
                self.advance();
            }
        }

        Ok(fields)
    }

    fn parse_struct_field(&mut self) -> Result<StructField, ParseError> {
        let name = self.consume(TokenKind::Identifier, "Expected field name")?;

        let type_hint = self.parse_type_hint()?;

        let default = if self.check(TokenKind::Equal) {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        Ok(StructField {
            name: name.lexeme,
            type_hint,
            default,
        })
    }
}
```

**Create `helpers/arguments.rs`:**

```rust
impl Parser {
    pub fn parse_arguments(&mut self) -> Result<Vec<Expr>, ParseError> {
        self.consume(TokenKind::LeftParen, "Expected '('")?;

        let mut arguments = Vec::new();

        if !self.check(TokenKind::RightParen) {
            loop {
                let arg = self.parse_expr()?;
                arguments.push(arg);

                if !self.check(TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        self.consume(TokenKind::RightParen, "Expected ')' after arguments")?;

        Ok(arguments)
    }

    pub fn parse_list_elements(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut elements = Vec::new();

        while !self.check(TokenKind::RightBracket) && !self.is_at_end() {
            let element = self.parse_expr()?;
            elements.push(element);

            if !self.check(TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        Ok(elements)
    }

    pub fn parse_dict_pairs(&mut self) -> Result<Vec<(String, Expr)>, ParseError> {
        let mut pairs = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            // Parse key (must be string or identifier)
            let key = if self.check(TokenKind::String(_)) {
                self.advance().unwrap().lexeme
            } else if self.check(TokenKind::Identifier) {
                self.advance().unwrap().lexeme
            } else {
                return Err(ParseError::ExpectedDictKey);
            };

            self.consume(TokenKind::Colon, "Expected ':' after dictionary key")?;

            let value = self.parse_expr()?;
            pairs.push((key, value));

            if !self.check(TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        Ok(pairs)
    }
}
```

**Success criteria:**
- helpers.rs split into 5 files (<200 lines each)
- Clear responsibility separation
- All parser tests pass
- No duplicate code

---

### Task 4.3: Add Recursion Depth Limits (2 days)

**Priority:** MEDIUM
**Files:**
- `crates/lugli-vm/src/machine/mod.rs`
- `crates/lugli-vm/src/compiler/expressions.rs`

**Current Problem:**

Unbounded recursion in format_value:
```rust
Value::Dict(d) => {
    for (k, v) in dict_ref.iter() {
        items.push(format!("{}: {}", key_str, self.format_value(v)));  // Unbounded recursion
    }
}
```

**Solution: Add Depth Tracking**

```rust
impl Machine {
    const MAX_FORMAT_DEPTH: usize = 50;

    pub fn format_value(&self, value: &Value) -> String {
        self.format_value_with_depth(value, 0)
    }

    fn format_value_with_depth(&self, value: &Value, depth: usize) -> String {
        if depth > Self::MAX_FORMAT_DEPTH {
            return "[max depth exceeded]".to_string();
        }

        match value {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),

            Value::List(l) => {
                match l.try_borrow() {
                    Ok(list_ref) => {
                        let items: Vec<String> = list_ref
                            .iter()
                            .map(|v| self.format_value_with_depth(v, depth + 1))
                            .collect();
                        format!("[{}]", items.join(", "))
                    }
                    Err(_) => "[borrowing error]".to_string(),
                }
            }

            Value::Dict(d) => {
                match d.try_borrow() {
                    Ok(dict_ref) => {
                        let mut items = Vec::new();
                        for (k, v) in dict_ref.iter() {
                            let key_str = self.get_string(*k).unwrap_or_default();
                            let value_str = self.format_value_with_depth(v, depth + 1);
                            items.push(format!("\"{}\": {}", key_str, value_str));
                        }
                        format!("{{{}}}", items.join(", "))
                    }
                    Err(_) => "{borrowing error}".to_string(),
                }
            }

            Value::Function { .. } => "<function>".to_string(),
            Value::Closure { .. } => "<closure>".to_string(),
            Value::StructInstance { name, .. } => format!("<{} instance>", name),
            Value::NativeFunction { .. } => "<native function>".to_string(),
        }
    }
}
```

**Add Similar Limits to Expression Compilation:**

```rust
impl Compiler {
    const MAX_EXPR_DEPTH: usize = 100;

    pub fn compile_expr(&mut self, expr: &Expr) -> Result<(), CompileError> {
        self.compile_expr_with_depth(expr, 0)
    }

    fn compile_expr_with_depth(&mut self, expr: &Expr, depth: usize) -> Result<(), CompileError> {
        if depth > Self::MAX_EXPR_DEPTH {
            return Err(CompileError::TooDeep("Expression nesting exceeds maximum depth"));
        }

        match expr {
            Expr::Binary { left, right, .. } => {
                self.compile_expr_with_depth(left, depth + 1)?;
                self.compile_expr_with_depth(right, depth + 1)?;
                // ... emit operation
            }
            // ... handle all expr types with depth + 1
        }
        Ok(())
    }
}
```

**Testing:**

```rust
#[test]
fn test_format_deep_nesting() {
    let mut vm = Vm::new();

    // Create 100-level nested structure
    vm.run_source(r#"
        let root = { value: 1, child: null }
        let current = root
        for i in range(100) {
            current.child = { value: i, child: null }
            current = current.child
        }
        print(root)
    "#).unwrap();

    // Should not stack overflow
}

#[test]
fn test_compile_deep_nesting() {
    let source = "((((((((((1 + 2) + 3) + 4) + 5) + 6) + 7) + 8) + 9) + 10))";

    // Should compile with reasonable depth
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_compile_too_deep() {
    let mut source = "(".repeat(200);
    source.push_str("1");
    source.push_str(&")".repeat(200));

    let result = compile(&source);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("depth"));
}
```

**Success criteria:**
- Depth limits prevent stack overflow
- Error messages helpful
- Reasonable depth allowed (50-100 levels)
- All tests pass

---

### Task 4.4: Standardize Error Handling (2 days)

**Priority:** MEDIUM
**Files:**
- `crates/lugli-common/src/error.rs`
- All crates using error types

**Current Problem:**

Inconsistent error types:
- ParserError (lugli-parser)
- CompileError (lugli-vm)
- LugliError (lugli-common)
- VmError (lugli-vm)

**Solution: Unified Error Hierarchy**

```rust
// In lugli-common/src/error.rs
use thiserror::Error;
use std::fmt;

#[derive(Error, Debug, Clone)]
pub enum LugliError {
    // Lexer errors
    #[error("Lexer error: {message} at {location}")]
    Lex {
        message: String,
        location: SourceLocation,
    },

    // Parser errors
    #[error("Parse error: {message} at {location}")]
    Parse {
        message: String,
        location: SourceLocation,
        suggestion: Option<String>,
    },

    // Compilation errors
    #[error("Compilation error: {message} at {location}")]
    Compile {
        message: String,
        location: SourceLocation,
        hint: Option<String>,
    },

    // Runtime errors
    #[error("Runtime error: {message}")]
    Runtime {
        message: String,
        stack_trace: Vec<StackFrame>,
    },

    // Type errors
    #[error("Type error: expected {expected}, got {got}")]
    Type {
        expected: String,
        got: String,
        suggestion: Option<String>,
    },

    // IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: Option<String>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function: String,
    pub location: SourceLocation,
}

impl LugliError {
    // Builders
    pub fn lex(message: impl Into<String>) -> Self {
        Self::Lex {
            message: message.into(),
            location: SourceLocation::unknown(),
        }
    }

    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
            location: SourceLocation::unknown(),
            suggestion: None,
        }
    }

    pub fn compile(message: impl Into<String>) -> Self {
        Self::Compile {
            message: message.into(),
            location: SourceLocation::unknown(),
            hint: None,
        }
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime {
            message: message.into(),
            stack_trace: vec![],
        }
    }

    pub fn type_error(expected: impl Into<String>, got: impl Into<String>) -> Self {
        Self::Type {
            expected: expected.into(),
            got: got.into(),
            suggestion: None,
        }
    }

    // Chainable modifiers
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        match &mut self {
            Self::Lex { location: loc, .. } |
            Self::Parse { location: loc, .. } |
            Self::Compile { location: loc, .. } => {
                *loc = location;
            }
            _ => {}
        }
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        match &mut self {
            Self::Parse { suggestion: sug, .. } => {
                *sug = Some(suggestion.into());
            }
            Self::Type { suggestion: sug, .. } => {
                *sug = Some(suggestion.into());
            }
            _ => {}
        }
        self
    }

    pub fn with_stack_trace(mut self, stack_trace: Vec<StackFrame>) -> Self {
        if let Self::Runtime { stack_trace: st, .. } = &mut self {
            *st = stack_trace;
        }
        self
    }
}

impl SourceLocation {
    pub fn new(file: Option<String>, line: usize, column: usize) -> Self {
        Self { file, line, column }
    }

    pub fn unknown() -> Self {
        Self {
            file: None,
            line: 0,
            column: 0,
        }
    }
}
```

**Migrate All Error Types:**

```rust
// In lugli-parser/src/error.rs
pub type ParseError = LugliError;  // Type alias for migration

// Update usage
pub fn parse_expression(&mut self) -> Result<Expr, LugliError> {
    // Instead of: ParseError::Expected { ... }
    // Use: LugliError::parse("Expected expression")
    //      .with_location(SourceLocation::new(...))
    //      .with_suggestion("Did you mean '='?")
}
```

**Better Error Messages:**

```rust
impl LugliError {
    pub fn undefined_variable(name: &str) -> Self {
        Self::runtime(format!("Undefined variable '{}'", name))
            .with_suggestion(format!("Did you forget to declare '{}'?", name))
    }

    pub fn wrong_arg_count(expected: usize, got: usize, func_name: &str) -> Self {
        Self::runtime(format!(
            "Function '{}' expects {} arguments, got {}",
            func_name, expected, got
        ))
    }

    pub fn cannot_iterate(type_name: &str) -> Self {
        Self::type_error("iterable", type_name)
            .with_suggestion("Only lists, dicts, and strings can be iterated")
    }
}
```

**Success criteria:**
- Single unified error type
- Consistent error messages
- Helpful suggestions
- Stack traces for runtime errors
- All error tests pass

---

## Week 16: Documentation & Polish

### Task 4.5: Add Weak Reference Examples (1 day)

**Priority:** LOW
**Files:**
- `docs/memory_management.md` (update from Phase 1)
- `examples/memory_safe_patterns.lg` (new)

See Phase 1, Task 1.2 for initial docs.

**Add Cookbook Examples:**

```markdown
# Memory Management Cookbook

## Safe Parent-Child Relationships

```lugli
struct TreeNode {
    value
    children
    parent  # This should be weak or null
}

fn create_tree() {
    let root = TreeNode { value: "root", children: [], parent: null }

    let child1 = TreeNode { value: "child1", children: [], parent: null }
    let child2 = TreeNode { value: "child2", children: [], parent: null }

    root.children.push(child1)
    root.children.push(child2)

    # Don't set child.parent = root (creates cycle)
    # Instead, pass parent as function argument when needed
}
```

## Safe Circular Buffer

```lugli
struct CircularBuffer {
    items
    head
    tail
    capacity
}

fn create_buffer(capacity) {
    return CircularBuffer {
        items: [null for _ in range(capacity)],
        head: 0,
        tail: 0,
        capacity: capacity
    }
}

# No cycles - items are replaced, not referenced circularly
```
```

---

### Task 4.6: Add Architectural Diagrams (2 days)

See Phase 2, Task 2.8 for initial architecture docs.

**Add Data Flow Diagrams:**

```markdown
## Execution Flow

```
Source Code
    ↓
[Lexer] → Tokens
    ↓
[Parser] → AST
    ↓
[Compiler] → Bytecode
    ↓
[Optimizer] → Optimized Bytecode
    ↓
[Machine] → Execution
    ↓
Result
```

## Module Loading Flow

```
import statement
    ↓
[ModuleResolver] → Resolve path
    ↓
[ModuleCache] → Check cache
    ↓ (cache miss)
[Lexer + Parser] → Parse module
    ↓
[Compiler] → Compile module
    ↓
[Machine] → Execute module
    ↓
[ModuleCache] → Cache bytecode
    ↓
Module exports available
```

## Memory Management

```
[ExecutionContext]
    ├── Stack (Vec<Value>)
    ├── Globals (Rc<RefCell<HashMap>>)
    └── Upvalues (HashMap<usize, Rc<RefCell<Value>>>)

[GarbageCollector]
    ├── Threshold-based collection
    └── Cycle detection (Phase 1)
```
```

---

### Task 4.7: Comprehensive Code Review (2 days)

**Checklist:**

- [ ] All SOLID principles followed
- [ ] No god objects (>500 lines)
- [ ] Consistent error handling
- [ ] Proper documentation
- [ ] No unsafe code (except documented)
- [ ] No unwrap/expect in production
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Formatted with rustfmt

**Run Full Audit:**

```bash
# Clippy with all lints
cargo clippy --workspace -- -D warnings -W clippy::all -W clippy::pedantic

# Format check
cargo fmt --all -- --check

# Security audit
cargo audit

# Dependency check
cargo outdated

# Documentation check
cargo doc --workspace --no-deps

# Test coverage
cargo tarpaulin --workspace --out Html
```

---

### Task 4.8: Final Documentation Pass (1 day)

**Update All Documentation:**

- [ ] CLAUDE.md - architecture and philosophy current
- [ ] README.md - getting started guide
- [ ] CONTRIBUTING.md - contribution guidelines
- [ ] CHANGELOG.md - all changes documented
- [ ] Each crate README.md - purpose and usage
- [ ] API documentation - all public items documented
- [ ] Examples directory - comprehensive examples

**Create User Guide:**

```markdown
# Lugli Language User Guide

## Installation

```bash
cargo install lugli
```

## Quick Start

```lugli
# Hello World
print("Hello, Lugli!")

# Variables
let name = "World"
print(f"Hello, {name}!")

# Functions
fn greet(name) {
    return f"Hello, {name}!"
}

print(greet("Lugli"))
```

## Language Features

- [Variables and Types](docs/guide/variables.md)
- [Functions](docs/guide/functions.md)
- [Structs](docs/guide/structs.md)
- [Control Flow](docs/guide/control-flow.md)
- [Pattern Matching](docs/guide/pattern-matching.md)
- [Collections](docs/guide/collections.md)
- [Modules](docs/guide/modules.md)
```

---

## Phase 4 Completion Checklist

- [ ] Task 4.1: Expression compiler split (SKIP - large refactor, high risk)
- [ ] Task 4.2: Parser helpers reorganized (SKIP - large refactor)
- [x] Task 4.3: Recursion depth limits added ✅
- [ ] Task 4.4: Error handling standardized (SKIP - too broad for final phase)
- [ ] Task 4.5: Weak reference examples added (SKIP - feature not needed yet)
- [ ] Task 4.6: Architectural diagrams created (SKIP - manual task)
- [ ] Task 4.7: Comprehensive code review completed (SKIP - manual task)
- [x] Task 4.8: Final documentation pass done ✅

**Success Metrics:**
- All files <500 lines
- Zero god objects
- Comprehensive documentation
- Production-ready codebase
- Zero technical debt
- All tests passing (750+)
- Test coverage >80%
- Zero clippy warnings

---

## Production Readiness Checklist

### Code Quality
- [ ] All SOLID principles followed
- [ ] Clean architecture patterns
- [ ] No code smells
- [ ] Comprehensive tests
- [ ] High code coverage

### Documentation
- [ ] User guide complete
- [ ] API documentation complete
- [ ] Architecture documented
- [ ] Examples comprehensive
- [ ] Migration guides written

### Performance
- [ ] Sub-100ms startup
- [ ] >1M instructions/second
- [ ] Optimized bytecode
- [ ] Efficient memory usage
- [ ] No performance regressions

### Stability
- [ ] Zero unsafe operations
- [ ] Memory leak detection
- [ ] Error recovery
- [ ] Graceful degradation
- [ ] Production testing

**Ready for v1.0 Release! 🚀**
