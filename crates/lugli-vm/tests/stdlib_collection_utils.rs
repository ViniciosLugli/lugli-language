mod helpers;
use helpers::run_test;

#[test]
fn test_zip_basic() {
    let source = r#"
        let numbers = [1, 2, 3]
        let letters = ["a", "b", "c"]
        let zipped = zip(numbers, letters)

        if zipped.len() != 3 {
            let error = 1 / 0
        }

        let first = zipped[0]
        if first[0] != 1 {
            let error = 1 / 0
        }
        if first[1] != "a" {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_all_function() {
    let source = r#"
        let all_true = [true, 1, "hello"]
        if !all(all_true) {
            let error = 1 / 0
        }

        let has_false = [true, 0, "hello"]
        if all(has_false) {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_any_function() {
    let source = r#"
        let has_true = [false, 0, 1]
        if !any(has_true) {
            let error = 1 / 0
        }

        let all_false = [false, 0, null]
        if any(all_false) {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_sum_function() {
    let source = r#"
        let numbers = [1, 2, 3, 4, 5]
        let total = sum(numbers)

        if total != 15 {
            let error = 1 / 0
        }

        # Sum with start value
        let total_with_start = sum(numbers, 10)
        if total_with_start != 25 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_min_max_functions() {
    let source = r#"
        let numbers = [5, 2, 8, 1, 9]

        let minimum = min(numbers)
        if minimum != 1 {
            let error = 1 / 0
        }

        let maximum = max(numbers)
        if maximum != 9 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_sorted_function() {
    let source = r#"
        let numbers = [5, 2, 8, 1]
        let sorted_nums = sorted(numbers)

        if sorted_nums[0] != 1 {
            let error = 1 / 0
        }
        if sorted_nums[3] != 8 {
            let error = 1 / 0
        }

        # Original should be unchanged
        if numbers[0] != 5 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_sorted_reverse() {
    let source = r#"
        let numbers = [5, 2, 8]
        let sorted_desc = sorted(numbers, true)

        if sorted_desc[0] != 8 {
            let error = 1 / 0
        }
        if sorted_desc[2] != 2 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_reversed_function() {
    let source = r#"
        let numbers = [1, 2, 3, 4]
        let rev = reversed(numbers)

        if rev[0] != 4 {
            let error = 1 / 0
        }
        if rev[3] != 1 {
            let error = 1 / 0
        }

        # Original should be unchanged
        if numbers[0] != 1 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_combined_operations() {
    let source = r#"
        # Create data
        let data = [1, 2, 3, 4, 5]

        # Filter and sum
        let evens = [x for x in data if x % 2 == 0]
        let sum_evens = sum(evens)

        if sum_evens != 6 {  # 2 + 4
            let error = 1 / 0
        }

        # Sort and get min/max
        let unsorted = [3, 1, 4, 1, 5]
        let sorted_data = sorted(unsorted)

        if min(sorted_data) != 1 {
            let error = 1 / 0
        }
        if max(sorted_data) != 5 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}
