mod helpers;
use helpers::run_test;

// get() tests
#[test]
fn test_dict_get_existing_key() {
    run_test(
        r#"
        let dict = {"name": "Alice", "age": 30}
        if dict.get("name") != "Alice" {
            let error = 1 / 0
        }
        if dict.get("age") != 30 {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_dict_get_missing_key() {
    run_test(
        r#"
        let dict = {"name": "Alice"}
        if dict.get("missing") != null {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_dict_get_with_default() {
    run_test(
        r#"
        let dict = {"name": "Alice"}
        let val = dict.get("missing", "default")
        if val != "default" {
            let error = 1 / 0
        }
    "#,
    );
}

// keys() tests
#[test]
fn test_dict_keys_basic() {
    run_test(
        r#"
        let dict = {"a": 1, "b": 2, "c": 3}
        let k = dict.keys()
        if k.len() != 3 {
            let error = 1 / 0
        }
        # Keys should contain all three keys
        if !k.contains("a") || !k.contains("b") || !k.contains("c") {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_dict_keys_empty() {
    run_test(
        r#"
        let dict = {}
        let k = dict.keys()
        if k.len() != 0 {
            let error = 1 / 0
        }
    "#,
    );
}

// values() tests
#[test]
fn test_dict_values_basic() {
    run_test(
        r#"
        let dict = {"a": 1, "b": 2, "c": 3}
        let v = dict.values()
        if v.len() != 3 {
            let error = 1 / 0
        }
        # Values should contain all three values
        if !v.contains(1) || !v.contains(2) || !v.contains(3) {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_dict_values_empty() {
    run_test(
        r#"
        let dict = {}
        let v = dict.values()
        if v.len() != 0 {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_dict_values_duplicate_values() {
    run_test(
        r#"
        let dict = {"a": 1, "b": 1, "c": 2}
        let v = dict.values()
        if v.len() != 3 {
            let error = 1 / 0
        }
    "#,
    );
}

// contains() tests
#[test]
fn test_dict_contains_key_exists() {
    run_test(
        r#"
        let dict = {"name": "Alice", "age": 30}
        if !dict.contains("name") {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_dict_contains_key_missing() {
    run_test(
        r#"
        let dict = {"name": "Alice"}
        if dict.contains("age") {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_dict_contains_empty_dict() {
    run_test(
        r#"
        let dict = {}
        if dict.contains("any") {
            let error = 1 / 0
        }
    "#,
    );
}

// len() tests
#[test]
fn test_dict_len_basic() {
    run_test(
        r#"
        let dict = {"a": 1, "b": 2, "c": 3}
        if dict.len() != 3 {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_dict_len_empty() {
    run_test(
        r#"
        let dict = {}
        if dict.len() != 0 {
            let error = 1 / 0
        }
    "#,
    );
}

#[test]
fn test_dict_len_after_assignment() {
    run_test(
        r#"
        let dict = {"a": 1}
        dict["b"] = 2
        dict["c"] = 3
        if dict.len() != 3 {
            let error = 1 / 0
        }
    "#,
    );
}

// Combined operations
#[test]
fn test_dict_combined_operations() {
    run_test(
        r#"
        let dict = {}
        dict["name"] = "Alice"
        dict["age"] = 30

        if dict.len() != 2 {
            let error = 1 / 0
        }

        if !dict.contains("name") {
            let error = 1 / 0
        }

        let val = dict.get("name")
        if val != "Alice" {
            let error = 1 / 0
        }

        let keys = dict.keys()
        if keys.len() != 2 {
            let error = 1 / 0
        }
    "#,
    );
}
