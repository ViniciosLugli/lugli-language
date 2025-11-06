# LUGLI TEST SUITE - MISSING TEST CASES

## Quick Reference: Tests That Need to Be Added

### STRING METHODS (0/11 have tests)

Each string method needs these test cases:
1. **upper()**
   - test_upper_basic() - "hello" → "HELLO"
   - test_upper_mixed() - "HeLLo" → "HELLO"
   - test_upper_empty() - "" → ""
   - test_upper_numbers() - "abc123" → "ABC123"
   - test_upper_unicode() - "café" → "CAFÉ"

2. **lower()**
   - test_lower_basic() - "HELLO" → "hello"
   - test_lower_mixed() - "HeLLo" → "hello"
   - test_lower_empty() - "" → ""
   - test_lower_unicode() - "CAFÉ" → "café"

3. **trim()**
   - test_trim_leading() - "  hello" → "hello"
   - test_trim_trailing() - "hello  " → "hello"
   - test_trim_both() - "  hello  " → "hello"
   - test_trim_empty() - "" → ""
   - test_trim_no_whitespace() - "hello" → "hello"

4. **chars()**
   - test_chars_basic() - "hello" → ["h","e","l","l","o"]
   - test_chars_empty() - "" → []
   - test_chars_unicode() - "café" → ["c","a","f","é"]
   - test_chars_numbers() - "123" → ["1","2","3"]

5. **split()**
   - test_split_default() - "hello world".split() → ["hello", "world"]
   - test_split_custom() - "a,b,c".split(",") → ["a", "b", "c"]
   - test_split_empty() - "".split(",") → [""]
   - test_split_single() - "hello".split(",") → ["hello"]
   - test_split_unicode() - "café;naïve".split(";") → ["café", "naïve"]

6. **contains()**
   - test_contains_found() - "hello".contains("ell") == true
   - test_contains_not_found() - "hello".contains("xyz") == false
   - test_contains_empty() - "hello".contains("") == true
   - test_contains_case_sensitive() - "Hello".contains("ell") == false

7. **starts_with()**
   - test_starts_with_true() - "hello".starts_with("he") == true
   - test_starts_with_false() - "hello".starts_with("ell") == false
   - test_starts_with_empty() - "hello".starts_with("") == true
   - test_starts_with_full() - "hello".starts_with("hello") == true

8. **ends_with()**
   - test_ends_with_true() - "hello".ends_with("lo") == true
   - test_ends_with_false() - "hello".ends_with("el") == false
   - test_ends_with_empty() - "hello".ends_with("") == true

9. **replace()**
   - test_replace_single() - "hello".replace("l", "L") → "heLLo"
   - test_replace_multiple() - "aaa".replace("a", "b") → "bbb"
   - test_replace_not_found() - "hello".replace("x", "y") → "hello"
   - test_replace_empty() - "hello".replace("", "x") → behavior?

10. **is_alphabetic()**
    - test_is_alphabetic_true() - "hello".is_alphabetic() == true
    - test_is_alphabetic_false() - "hello123".is_alphabetic() == false
    - test_is_alphabetic_empty() - "".is_alphabetic() == false
    - test_is_alphabetic_unicode() - "café".is_alphabetic() == true

11. **join()** (reverse of split)
    - test_join_basic() - ["a","b","c"].join(",") → "a,b,c"
    - test_join_empty() - [].join(",") → ""
    - test_join_single() - ["hello"].join(",") → "hello"

---

### LIST METHODS (3/11 have tests)

1. **push()** - PARTIALLY TESTED
   - test_push_single() - [1,2,3].push(4) → [1,2,3,4]
   - test_push_multiple() - Test 100 pushes
   - test_push_mixed_types() - [1,"a"].push(true)
   - test_push_returns_null() - push() return value

2. **pop()** - PARTIALLY TESTED
   - test_pop_normal() - [1,2,3].pop() → 3
   - test_pop_empty() - [].pop() → null
   - test_pop_single() - [42].pop() → 42
   - test_pop_multiple() - Verify list shrinks

3. **append()** - NO TESTS
   - test_append_basic() - [1,2].append([3,4]) → [1,2,3,4]
   - test_append_empty_to() - [].append([1,2]) → [1,2]
   - test_append_to_empty() - [1,2].append([]) → [1,2]
   - test_append_self() - Should error (circular)

4. **get()** - NO TESTS
   - test_get_valid() - list.get(1) → second element
   - test_get_out_of_bounds() - list.get(100) → null
   - test_get_negative() - list.get(-1) → error?
   - test_get_non_integer() - list.get(1.5) → error

5. **set()** - NO TESTS
   - test_set_valid() - list.set(0, 99)
   - test_set_out_of_bounds() - list.set(100, 99) → error?
   - test_set_negative() - list.set(-1, 99) → error?

6. **is_empty()** - NO TESTS
   - test_is_empty_true() - [].is_empty() == true
   - test_is_empty_false() - [1].is_empty() == false

7. **clear()** - NO TESTS
   - test_clear_basic() - [1,2,3].clear() → []
   - test_clear_empty() - [].clear() → []

8. **reverse()** - NO TESTS
   - test_reverse_basic() - [1,2,3].reverse() → [3,2,1]
   - test_reverse_empty() - [].reverse() → []
   - test_reverse_single() - [42].reverse() → [42]

9. **filter()** - PARTIALLY TESTED
   - test_filter_closure() - [1,2,3,4].filter(fn(x) { x > 2 }) → [3,4]
   - test_filter_empty() - [].filter(...) → []
   - test_filter_all_match() - All pass filter
   - test_filter_none_match() - All fail filter

10. **map!()** - PARTIALLY TESTED
    - test_map_closure() - [1,2,3].map!(fn(x) { x*2 }) → [2,4,6]
    - test_map_empty() - [].map!(...) → []
    - test_map_type_change() - Transform types

11. **sorted()** - NO TESTS
    - test_sorted_numbers() - [3,1,2].sorted() → [1,2,3]
    - test_sorted_strings() - ["c","a","b"].sorted() → ["a","b","c"]
    - test_sorted_empty() - [].sorted() → []

12. **reversed()** - NO TESTS
    - test_reversed_basic() - [1,2,3].reversed() → [3,2,1]
    - test_reversed_empty() - [].reversed() → []

---

### DICTIONARY METHODS (0/5 have tests)

1. **get()**
   - test_get_found() - dict.get("key") → value
   - test_get_not_found() - dict.get("missing") → error
   - test_get_default() - dict.get("missing", "default") → "default"

2. **keys()**
   - test_keys_basic() - {a:1, b:2}.keys() → list of keys
   - test_keys_empty() - {}.keys() → []

3. **values()**
   - test_values_basic() - {a:1, b:2}.values() → list of values
   - test_values_empty() - {}.values() → []

4. **contains()**
   - test_contains_key() - dict.contains("key") == true
   - test_contains_no_key() - dict.contains("missing") == false

5. **len()**
   - test_len_basic() - len({a:1, b:2}) == 2
   - test_len_empty() - len({}) == 0

---

### ASSIGNMENT OPERATORS (0/5 have focused tests)

Each operator needs edge cases:

1. **+= (add assign)**
   - test_add_assign_numbers() - x += 5
   - test_add_assign_strings() - s += "!"
   - test_add_assign_lists() - l += [1,2]
   - test_add_assign_chain() - x += y += z
   - test_add_assign_undefined() - Should error

2. **-= (subtract assign)**
   - test_sub_assign_basic() - x -= 5
   - test_sub_assign_negative() - x -= -5
   - test_sub_assign_zero() - x -= 0

3. **\*= (multiply assign)**
   - test_mul_assign_basic() - x *= 5
   - test_mul_assign_zero() - x *= 0
   - test_mul_assign_one() - x *= 1

4. **/= (divide assign)**
   - test_div_assign_basic() - x /= 5
   - test_div_assign_by_one() - x /= 1
   - test_div_assign_by_zero() - Should error

5. **%= (modulo assign)**
   - test_mod_assign_basic() - x %= 3
   - test_mod_assign_by_zero() - Should error
   - test_mod_assign_negative() - x %= -3

---

### EDGE CASES (0/20 have tests)

1. **Empty Collections**
   - test_empty_string_len() - "".len() == 0
   - test_empty_list_len() - [].len() == 0
   - test_empty_dict_len() - {}.len() == 0
   - test_empty_string_indexing() - ""[0] → error
   - test_empty_list_indexing() - [][0] → error

2. **Null Handling**
   - test_null_equals_null() - null == null
   - test_null_not_equals() - null != 5
   - test_null_truthiness() - if null { } → false
   - test_null_in_operations() - null + 5 → error?
   - test_null_method_call() - null.upper() → error

3. **Type Mismatches**
   - test_list_plus_string() - [1] + "x" → error?
   - test_string_times_number() - "x" * 3 → error?
   - test_index_with_string() - [1,2,3]["0"] → error?

4. **Out of Bounds**
   - test_positive_out_of_bounds() - [1,2,3][10] → error
   - test_large_negative_index() - [1,2,3][-100] → error
   - test_string_out_of_bounds() - "hello"[10] → error?

5. **Number Edge Cases**
   - test_very_large_number() - 1e308
   - test_division_by_infinity() - x / inf
   - test_nan_equality() - nan == nan → false?

---

### ERROR CONDITIONS (0/15 have tests)

1. **Invalid Method Calls**
   - test_method_on_number() - 42.upper() → error
   - test_method_on_null() - null.push(1) → error
   - test_nonexistent_method() - "x".nonexistent() → error

2. **Type Errors**
   - test_add_incompatible() - "x" + [] → error?
   - test_compare_incompatible() - "x" > [] → error?

3. **Index Errors**
   - test_index_non_integer() - [1,2,3][1.5] → error
   - test_index_string_on_list() - [1,2,3]["a"] → error
   - test_index_non_indexable() - 42[0] → error

4. **Argument Errors**
   - test_function_too_many_args() - len(1,2,3) → error
   - test_function_too_few_args() - len() → error

---

## TOTAL COUNT

Missing tests: **150+ test cases**
- String methods: 45 test cases
- List methods: 35 test cases
- Dict methods: 15 test cases
- Assignment operators: 20 test cases
- Edge cases: 20 test cases
- Error conditions: 15 test cases

These should be organized into:
- `crates/lugli-stdlib/tests/string_methods_tests.rs` (45 tests)
- `crates/lugli-stdlib/tests/list_methods_tests.rs` (35 tests)
- `crates/lugli-stdlib/tests/dict_methods_tests.rs` (15 tests)
- `crates/lugli-vm/tests/assignment_operators_tests.rs` (20 tests)
- `crates/lugli-vm/tests/edge_cases_tests.rs` (20 tests)
- `crates/lugli-vm/tests/error_conditions_tests.rs` (15 tests)

