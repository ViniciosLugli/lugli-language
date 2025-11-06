mod helpers;
use helpers::run_test;

#[test]
fn test_newline_in_print() {
    let source = r#"
        let s = "hello\nworld"
        print(s)
    "#;

    run_test(source);
}

#[test]
fn test_tab_formatting() {
    let source = r#"
        let s = "Name:\tAlice\nAge:\t30"
        if !s.contains("\t") {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_escape_in_comparison() {
    let source = r#"
        let s1 = "hello\nworld"
        let s2 = "hello\nworld"

        if s1 != s2 {
            let error = 1 / 0
        }

        let s3 = "hello\\nworld"
        if s1 == s3 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_escaped_quotes_in_dict() {
    let source = r#"
        let d = {
            "say": "He said \"hello\""
        }

        if !d["say"].contains("\"") {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_multiline_string_split() {
    let source = r#"
        let s = "line1\nline2\nline3"
        let lines = s.split("\n")

        if lines.len() != 3 {
            let error = 1 / 0
        }

        if lines[0] != "line1" {
            let error = 1 / 0
        }

        if lines[2] != "line3" {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_windows_path_operations() {
    let source = r#"
        let path = "C:\\Users\\Alice\\file.txt"

        if !path.contains("\\") {
            let error = 1 / 0
        }

        let parts = path.split("\\")
        if parts.len() != 4 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_escape_in_string_methods() {
    let source = r#"
        let s = "test\nstring"

        if !s.contains("\n") {
            let error = 1 / 0
        }

        if !s.starts_with("test") {
            let error = 1 / 0
        }

        if !s.ends_with("string") {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_escape_length_calculation() {
    let source = r#"
        let s1 = "test"
        let s2 = "test\n"

        if s1.len() != 4 {
            let error = 1 / 0
        }

        if s2.len() != 5 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_escape_in_fstring() {
    let source = r#"
        let name = "Alice"
        let message = f"Hello\n{name}"

        if !message.contains("\n") {
            let error = 1 / 0
        }

        if !message.contains("Alice") {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_replace_with_escapes() {
    let source = r#"
        let s = "hello world"
        let replaced = s.replace(" ", "\n")

        if !replaced.contains("\n") {
            let error = 1 / 0
        }

        if replaced.split("\n").len() != 2 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_escape_in_list_and_dict() {
    let source = r#"
        let items = ["line1\nline2", "line3\nline4"]

        if items[0].split("\n").len() != 2 {
            let error = 1 / 0
        }

        let data = {
            "key\n1": "value\n1"
        }

        if data.keys().len() != 1 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}
