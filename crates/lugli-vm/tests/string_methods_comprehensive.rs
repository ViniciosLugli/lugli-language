mod helpers;
use helpers::run_test;

// upper() tests
#[test]
fn test_string_upper_basic() {
    run_test(
        r#"
        let s = "hello"
        if s.upper() != "HELLO" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_upper_already_upper() {
    run_test(
        r#"
        let s = "HELLO"
        if s.upper() != "HELLO" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_upper_mixed_case() {
    run_test(
        r#"
        let s = "HeLLo WoRLd"
        if s.upper() != "HELLO WORLD" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_upper_empty() {
    run_test(
        r#"
        let s = ""
        if s.upper() != "" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_upper_numbers() {
    run_test(
        r#"
        let s = "hello123"
        if s.upper() != "HELLO123" {
            let error = 1 / 0
        }
    "#,
    );
}

// lower() tests
#[test]
fn test_string_lower_basic() {
    run_test(
        r#"
        let s = "HELLO"
        if s.lower() != "hello" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_lower_already_lower() {
    run_test(
        r#"
        let s = "hello"
        if s.lower() != "hello" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_lower_mixed_case() {
    run_test(
        r#"
        let s = "HeLLo WoRLd"
        if s.lower() != "hello world" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_lower_empty() {
    run_test(
        r#"
        let s = ""
        if s.lower() != "" {
            let error = 1 / 0
        }
    "#,
    );
}

// trim() tests
#[test]
fn test_string_trim_basic() {
    run_test(
        r#"
        let s = "  hello  "
        if s.trim() != "hello" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_trim_leading_only() {
    run_test(
        r#"
        let s = "  hello"
        if s.trim() != "hello" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_trim_trailing_only() {
    run_test(
        r#"
        let s = "hello  "
        if s.trim() != "hello" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_trim_no_whitespace() {
    run_test(
        r#"
        let s = "hello"
        if s.trim() != "hello" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_trim_empty() {
    run_test(
        r#"
        let s = ""
        if s.trim() != "" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_trim_only_whitespace() {
    run_test(
        r#"
        let s = "   "
        if s.trim() != "" {
            let error = 1 / 0
        }
    "#,
    );
}

// split() tests
#[test]
fn test_string_split_basic() {
    run_test(
        r#"
        let s = "a,b,c"
        let parts = s.split(",")
        if parts.len() != 3 {
            let error = 1 / 0
        }
        if parts[0] != "a" || parts[1] != "b" || parts[2] != "c" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_split_single_char() {
    run_test(
        r#"
        let s = "hello"
        let parts = s.split("l")
        if parts.len() != 3 {
            let error = 1 / 0
        }
        if parts[0] != "he" || parts[1] != "" || parts[2] != "o" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_split_no_delimiter() {
    run_test(
        r#"
        let s = "hello"
        let parts = s.split(",")
        if parts.len() != 1 {
            let error = 1 / 0
        }
        if parts[0] != "hello" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_split_empty_string() {
    run_test(
        r#"
        let s = ""
        let parts = s.split(",")
        if parts.len() != 1 {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_split_consecutive_delimiters() {
    run_test(
        r#"
        let s = "a,,b"
        let parts = s.split(",")
        if parts.len() != 3 {
            let error = 1 / 0
        }
        if parts[1] != "" {
            let error = 1 / 0
        }
    "#,
    );
}

// contains() tests
#[test]
fn test_string_contains_found() {
    run_test(
        r#"
        let s = "hello world"
        if !s.contains("world") {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_contains_not_found() {
    run_test(
        r#"
        let s = "hello world"
        if s.contains("xyz") {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_contains_empty_substring() {
    run_test(
        r#"
        let s = "hello"
        if !s.contains("") {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_contains_full_string() {
    run_test(
        r#"
        let s = "hello"
        if !s.contains("hello") {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_contains_case_sensitive() {
    run_test(
        r#"
        let s = "Hello World"
        if s.contains("hello") {
            let error = 1 / 0
        }
    "#,
    );
}

// starts_with() tests
#[test]
fn test_string_starts_with_true() {
    run_test(
        r#"
        let s = "hello world"
        if !s.starts_with("hello") {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_starts_with_false() {
    run_test(
        r#"
        let s = "hello world"
        if s.starts_with("world") {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_starts_with_empty() {
    run_test(
        r#"
        let s = "hello"
        if !s.starts_with("") {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_starts_with_full_string() {
    run_test(
        r#"
        let s = "hello"
        if !s.starts_with("hello") {
            let error = 1 / 0
        }
    "#,
    );
}

// ends_with() tests
#[test]
fn test_string_ends_with_true() {
    run_test(
        r#"
        let s = "hello world"
        if !s.ends_with("world") {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_ends_with_false() {
    run_test(
        r#"
        let s = "hello world"
        if s.ends_with("hello") {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_ends_with_empty() {
    run_test(
        r#"
        let s = "hello"
        if !s.ends_with("") {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_ends_with_full_string() {
    run_test(
        r#"
        let s = "hello"
        if !s.ends_with("hello") {
            let error = 1 / 0
        }
    "#,
    );
}

// replace() tests
#[test]
fn test_string_replace_basic() {
    run_test(
        r#"
        let s = "hello world"
        if s.replace("world", "there") != "hello there" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_replace_multiple() {
    run_test(
        r#"
        let s = "foo bar foo"
        if s.replace("foo", "baz") != "baz bar baz" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_replace_no_match() {
    run_test(
        r#"
        let s = "hello world"
        if s.replace("xyz", "abc") != "hello world" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_replace_empty_search() {
    run_test(
        r#"
        let s = "hello"
        let result = s.replace("", "x")
        # Empty replacement has implementation-specific behavior
    "#,
    );
}

// len() tests
#[test]
fn test_string_len_basic() {
    run_test(
        r#"
        let s = "hello"
        if s.len() != 5 {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_len_empty() {
    run_test(
        r#"
        let s = ""
        if s.len() != 0 {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_len_with_spaces() {
    run_test(
        r#"
        let s = "hello world"
        if s.len() != 11 {
            let error = 1 / 0
        }
    "#,
    );
}

// chars() tests
#[test]
fn test_string_chars_basic() {
    run_test(
        r#"
        let s = "abc"
        let c = s.chars()
        if c.len() != 3 {
            let error = 1 / 0
        }
        if c[0] != "a" || c[1] != "b" || c[2] != "c" {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_chars_empty() {
    run_test(
        r#"
        let s = ""
        let c = s.chars()
        if c.len() != 0 {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_string_chars_single() {
    run_test(
        r#"
        let s = "x"
        let c = s.chars()
        if c.len() != 1 {
            let error = 1 / 0
        }
        if c[0] != "x" {
            let error = 1 / 0
        }
    "#,
    );
}
