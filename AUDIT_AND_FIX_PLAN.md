# Lugli Language - Comprehensive Audit & Fix Plan
**Date:** 2025-11-06
**Branch:** `claude/audit-tests-bugs-fixes-011CUqnY8wEnYvuDNvvEJKVS`
**Current Status:** 787 tests passing (100%), multiple critical bugs identified

---

## 📊 Executive Summary

**Agents Deployed:** 3 (QA Engineer, Senior Debugger, System Architect)
**Investigation Scope:** Test coverage, bug hunting, feature completeness
**Key Findings:**
- ✅ **Strong foundation**: 787 passing tests, stable core language
- ❌ **2 CRITICAL bugs**: Float index truncation, escape sequences not processed
- ⚠️ **Major test gaps**: 150+ missing edge case tests (stdlib methods untested)
- 📋 **9 incomplete features**: Ranging from partially implemented to broken

**Recommendation:** Fix critical bugs → Add edge case tests → Complete/fix partial features

---

## 🚨 CRITICAL BUGS (Must Fix Immediately)

### 1. **Float Index Truncation - Silent Data Corruption**
**Severity:** CRITICAL
**File:** `crates/lugli-vm/src/machine.rs:1463`
**Impact:** Silently truncates float indices instead of rejecting them

**Current Behavior:**
```lugli
let arr = [10, 20, 30]
arr[1.5]        # Returns 20 (WRONG - should error)
arr.get(1.5)    # Throws error (CORRECT)
```

**Root Cause:**
```rust
// Line 1463 - direct cast without validation
let idx_i64 = *idx as i64;
```

**Fix Required:**
```rust
// Add fractional check before casting
if idx.fract() != 0.0 {
    return Err(LugliError::runtime(format!("Index must be an integer, got {}", idx)));
}
let idx_i64 = *idx as i64;
```

**Test Cases to Add:**
- `arr[1.5]` → error
- `arr[1.9999]` → error
- `arr[-1.5]` → error
- `arr[1.0]` → success (exact integer)

---

### 2. **String Escape Sequences Not Processed**
**Severity:** CRITICAL
**File:** `crates/lugli-parser/src/literals.rs:110`
**Impact:** Cannot use `\n`, `\t`, `\r`, `\\`, `\"` in strings

**Current Behavior:**
```lugli
let s = "hello\nworld"
print(s)  # Outputs: hello\nworld (literal backslash-n)
```

**Expected:**
```lugli
let s = "hello\nworld"
print(s)  # Should output:
          # hello
          # world
```

**Root Cause:**
```rust
// Line 110 - only strips quotes, doesn't unescape
let unquoted_value = s[1..s.len() - 1].to_string();
```

**Fix Required:**
Implement `unescape_string()` function to process:
- `\n` → newline
- `\t` → tab
- `\r` → carriage return
- `\\` → backslash
- `\"` → quote
- `\'` → single quote

**Test Cases to Add:**
- Basic escapes: `"\n"`, `"\t"`, `"\r"`
- Multiple escapes: `"line1\nline2\nline3"`
- Escaped quotes: `"He said \"hello\""`
- Backslash escaping: `"C:\\path\\to\\file"`
- Invalid escapes: `"\x"` → error or literal

---

## 🔴 HIGH PRIORITY BUGS

### 3. **Inconsistent Out-of-Bounds Handling**
**Severity:** HIGH
**Files:** `crates/lugli-vm/src/machine.rs:1475-1476`, `crates/lugli-stdlib/src/core/list.rs:142`
**Impact:** Confusing API - same operation has different error semantics

**Inconsistencies:**

**List Access:**
- `arr[10]` → throws error ❌
- `arr.get(10)` → returns `null` ✓

**Dictionary Access:**
- `dict["missing"]` → returns `null` ✓
- `dict.get("missing")` → throws error ❌

**Recommended Fix:** Standardize to return `null` for missing keys/indices in ALL cases:
- `arr[10]` → `null`
- `arr.get(10)` → `null`
- `dict["missing"]` → `null`
- `dict.get("missing")` → `null`

**Rationale:** Matches Python/JavaScript semantics, avoids defensive coding

**Test Cases to Add:**
- List bracket notation out-of-bounds → `null`
- List.get() out-of-bounds → `null`
- Dict bracket notation missing key → `null`
- Dict.get() missing key → `null`
- Consistency tests comparing both syntaxes

---

### 4. **Negative Index Handling Inconsistency**
**Severity:** MEDIUM
**File:** `crates/lugli-vm/src/machine.rs:1465-1473`
**Impact:** Confusing API design

**Current Behavior:**
- `arr[-1]` → works (returns last element) ✓
- `arr.get(-1)` → throws "Negative index -1 not allowed" ❌

**Recommended Fix:** Support negative indices in ALL list methods:
- `list.get(-1)` → last element
- `list.set(-1, value)` → update last element

**Test Cases to Add:**
- `arr.get(-1)`, `arr.get(-2)`, etc.
- `arr.set(-1, value)`
- Negative index out of bounds: `arr.get(-100)` → `null`

---

## 📋 TEST COVERAGE GAPS (150+ Missing Tests)

### **String Methods (45 tests needed)**
**Status:** 11 methods with ZERO dedicated tests

**Missing Tests:**
- `upper()`: basic, unicode, empty string
- `lower()`: basic, unicode, mixed case
- `trim()`: leading/trailing, internal whitespace, empty
- `chars()`: ascii, unicode, empty
- `split()`: basic, delimiter variations, empty string, no delimiter found
- `contains()`: found, not found, empty string, empty substring
- `starts_with()`: true, false, empty cases
- `ends_with()`: true, false, empty cases
- `replace()`: single, multiple, no match, empty strings
- `is_alphabetic()`: true, false, mixed, unicode
- `join()`: basic list, empty list, single item

**Edge Cases:**
- Empty strings as input
- Unicode characters
- Very long strings (>1MB)
- Null/undefined inputs

---

### **List Methods (35 tests needed)**
**Status:** 8+ methods with zero/limited tests

**Missing Tests:**
- `append()`: basic, empty list, chaining
- `get()`: valid index, out of bounds, negative, float (should error)
- `set()`: valid, out of bounds, negative, float (should error)
- `is_empty()`: true, false
- `clear()`: basic, already empty, verify length=0
- `reverse()`: basic, single item, empty, odd/even length
- `sorted()`: ascending, descending, duplicates, single, empty
- `reversed()`: vs reverse() (mutating vs non-mutating)

**Edge Cases:**
- Empty lists
- Single-element lists
- Very large lists (>10k items)
- Nested lists
- Type mixing

---

### **Dictionary Methods (15 tests needed)**
**Status:** 5 methods untested

**Missing Tests:**
- `get()`: existing key, missing key, null key
- `keys()`: basic, empty dict, order preservation
- `values()`: basic, empty dict, duplicate values
- `contains()`: true, false, null key
- `len()`: basic, empty, after modifications

**Edge Cases:**
- Empty dictionaries
- Numeric vs string keys
- Key collision with floats (1.0 vs 1)

---

### **Assignment Operators (20 tests needed)**
**Status:** Critically undertested (+=, -=, *=, /=, %=)

**Missing Tests:**
```lugli
# All combinations of:
x += 5   # Add and assign
x -= 3   # Subtract and assign
x *= 2   # Multiply and assign
x /= 4   # Divide and assign
x %=  3   # Modulo and assign
```

**Test Coverage Needed:**
- Basic operation for each operator
- Chaining: `x += 1; x += 2; x += 3`
- Type coercion: `x = "hello"; x += " world"`
- Error cases: division by zero with `/=`
- List/dict operations: `list += [item]`, `dict += other_dict`

---

### **Control Flow Edge Cases (15 tests needed)**

**Missing:**
- Empty match blocks
- Match with all guards failing
- Nested loops with break/continue
- For loops with empty iterables
- While loops that never execute
- Loop with immediate break

---

### **Error Handling Tests (20 tests needed)**

**Missing:**
- Type errors in operations
- Null/undefined handling
- Stack overflow (deep recursion)
- Out-of-memory scenarios
- Invalid function arguments
- Parse error recovery

---

## 🔧 UNIMPLEMENTED/PARTIALLY IMPLEMENTED FEATURES

### 5. **Module/Import System (Partially Working)**
**Status:** Parsed, compiled, infrastructure exists, but not fully functional
**Files:** `crates/lugli-vm/src/compiler/statements.rs:281-318`, `crates/lugli-vm/src/module.rs`

**What Works:**
- Parser accepts `import module`, `from module import items`
- ModuleResolver, ModuleCache, Module infrastructure
- VM instructions: `ImportModule`, `ImportFrom`

**What's Broken:**
- File-based module loading fails silently
- Stdlib modules not pre-loaded
- No tests for import functionality

**Fix Plan:**
1. Add tests for basic imports
2. Pre-load stdlib modules at VM initialization
3. Fix file-based module resolution
4. Add error messages for missing modules

**Test Cases to Add:**
- `import math` → access `math.pi`
- `from stdlib import List` → use List
- Circular import detection
- Module not found errors

---

### 6. **map(), filter(), reduce() Functions (Declared but Unusable)**
**Status:** Returns user error with workaround suggestion
**File:** `crates/lugli-stdlib/src/core/list.rs:179-191`

**Current Implementation:**
```rust
pub fn list_map(_args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    Err(LugliError::runtime(
        "map() not yet implemented. Use list comprehensions: [f(x) for x in list]"
    ))
}
```

**Why Not Implemented:** Requires passing functions as arguments and calling them in VM context

**Options:**
- **Option A:** Implement proper first-class function support
- **Option B:** Remove from stdlib (list comprehensions already work)
- **Option C:** Keep error with helpful message (current state)

**Recommendation:** Keep current state - list comprehensions are more Pythonic

---

### 7. **Nested F-Strings (Not Supported)**
**Status:** Parser explicitly rejects
**File:** `crates/lugli-parser/tests/fstring_edge_cases.rs:62-69`

**Example:**
```lugli
f"outer {f"inner {x}"}"  # Parser error
```

**Documented As:** v0.4.0 planned feature

**Fix Plan:**
1. Update f-string parser to track nesting depth
2. Recursively parse inner f-strings
3. Emit nested bytecode
4. Add tests for 2-3 levels of nesting

**Test Cases to Add:**
- Single-level nesting
- Multi-level nesting
- Max depth limits

---

### 8. **Index Assignment (Not Supported)**
**Status:** Language-level limitation
**Files:** `examples/samples/hangman.lg:36`, parser tests

**What Doesn't Work:**
```lugli
list[0] = value       # Not supported
dict["key"] = value   # Not supported
```

**Current Workaround:**
```lugli
list.set(0, value)    # Use method instead
```

**Fix Plan:**
1. Add parser support for index assignment as statement
2. Emit `SetIndex` VM instruction
3. Update compiler to handle assignment target
4. Add comprehensive tests

**Test Cases to Add:**
- `arr[0] = 10`
- `dict["key"] = "value"`
- Chaining: `arr[0] = arr[1] = 10`
- Error: `arr[10] = 5` (out of bounds)

---

### 9. **Impl Blocks (Parsed but Broken)**
**Status:** Parser accepts, but runtime errors
**File:** `crates/lugli-parser/src/statements.rs`

**Problem:** Parser converts `impl StructName { ... }` to `StructDecl`, but method suffix parsing (!) conflicts with field parsing

**Example:**
```lugli
impl Counter {
    fn increment!(self) {  # Parser error on '!'
        self.value += 1
    }
}
```

**Fix Plan:**
1. Fix method suffix parsing in impl blocks
2. Ensure proper conversion to StructDecl
3. Test with both regular and suffixed methods

**Test Cases to Add:**
- Basic impl block
- Methods with `!` suffix
- Methods with `?` suffix
- Mixed methods and fields

---

## 📈 IMPLEMENTATION PRIORITY

### **Phase 1: Critical Bug Fixes (Day 1)**
**Time Estimate:** 4-6 hours

1. ✅ Fix float index truncation (`machine.rs:1463`)
2. ✅ Implement escape sequence processing (`literals.rs:110`)
3. ✅ Add tests for both fixes
4. ✅ Run full test suite
5. ✅ Commit: "fix: Add float index validation and escape sequence processing"

**Success Criteria:**
- `arr[1.5]` throws error
- `"hello\nworld"` prints with newline
- All 787 tests still pass
- New edge case tests pass

---

### **Phase 2: High Priority Bugs (Day 1-2)**
**Time Estimate:** 4-6 hours

1. ✅ Standardize out-of-bounds handling (all → `null`)
2. ✅ Add negative index support to list methods
3. ✅ Add 30+ edge case tests
4. ✅ Run full test suite
5. ✅ Commit: "fix: Standardize out-of-bounds and negative index handling"

**Success Criteria:**
- Consistent behavior across bracket notation and methods
- All edge cases tested
- No breaking changes to existing tests

---

### **Phase 3: Test Coverage (Day 2-3)**
**Time Estimate:** 8-10 hours

1. ✅ Add 45 string method tests
2. ✅ Add 35 list method tests
3. ✅ Add 15 dict method tests
4. ✅ Add 20 assignment operator tests
5. ✅ Add 20 control flow edge case tests
6. ✅ Add 20 error handling tests
7. ✅ Run full test suite (target: 950+ tests)
8. ✅ Commit: "test: Add comprehensive edge case coverage for stdlib"

**Success Criteria:**
- 150+ new test cases
- 85%+ coverage for stdlib methods
- All tests pass
- Test execution time <15 seconds

---

### **Phase 4: Feature Completion (Day 3-4)**
**Time Estimate:** 10-12 hours

1. ✅ Fix module/import system
2. ✅ Implement nested f-strings
3. ✅ Add index assignment support
4. ✅ Fix impl blocks
5. ✅ Add tests for each feature
6. ✅ Run full test suite
7. ✅ Commit: "feat: Complete module imports, nested f-strings, index assignment, impl blocks"

**Success Criteria:**
- All 4 features working
- Comprehensive tests for each
- Documentation updated
- All tests pass (target: 1000+ tests)

---

### **Phase 5: Polish & Refactor (Day 4-5)**
**Time Estimate:** 4-6 hours

1. ✅ Remove outdated TODOs/FIXMEs
2. ✅ Update CLAUDE.md with accurate feature status
3. ✅ Run benchmarks, check for regressions
4. ✅ Final test suite run
5. ✅ Commit: "chore: Update documentation and remove deprecated code"

**Success Criteria:**
- CLAUDE.md matches implementation
- No performance regressions
- All tests pass
- Codebase is clean and consistent

---

## ✅ Success Criteria (Final)

**Before pushing:**
- ✅ All critical bugs fixed
- ✅ All high priority bugs fixed
- ✅ 150+ new edge case tests added
- ✅ Test count: 950+ (from 787)
- ✅ Test pass rate: 100%
- ✅ 4 major features completed
- ✅ Documentation updated
- ✅ No performance regressions
- ✅ Codebase is clean, consistent, production-ready

**Final Deliverables:**
1. All fixes committed to `claude/audit-tests-bugs-fixes-011CUqnY8wEnYvuDNvvEJKVS`
2. Updated CLAUDE.md
3. Test coverage report
4. Performance benchmark results
5. Ready for PR to main branch

---

## 📝 Notes

**Agent Reports Available:**
- `TEST_AUDIT_REPORT.md` - Detailed test coverage analysis
- `MISSING_TESTS.md` - Quick reference for 150+ missing test cases
- `TEST_COVERAGE_SUMMARY.md` - Coverage matrix and metrics

**Estimated Total Time:** 30-40 hours over 4-5 days

**Risk Assessment:**
- **Low Risk:** Bug fixes, test additions
- **Medium Risk:** Feature completions (may introduce regressions)
- **High Risk:** Module system changes (affects core architecture)

**Mitigation:** Run full test suite after each phase, commit frequently
