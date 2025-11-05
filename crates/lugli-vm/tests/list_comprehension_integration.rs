mod helpers;
use helpers::run_test;

#[test]
fn test_basic_list_comprehension() {
    let source = r#"
        let numbers = [1, 2, 3, 4, 5]
        let doubled = [x * 2 for x in numbers]

        if doubled.len() != 5 {
            let error = 1 / 0
        }

        if doubled[0] != 2 {
            let error = 1 / 0
        }

        if doubled[4] != 10 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_list_comprehension_with_filter() {
    let source = r#"
        let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        let evens = [x for x in numbers if x % 2 == 0]

        if evens.len() != 5 {
            let error = 1 / 0
        }

        if evens[0] != 2 {
            let error = 1 / 0
        }

        if evens[4] != 10 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_list_comprehension_complex_expression() {
    let source = r#"
        let numbers = [1, 2, 3, 4, 5]
        let transformed = [x * x + 1 for x in numbers]

        # 1*1+1=2, 2*2+1=5, 3*3+1=10, 4*4+1=17, 5*5+1=26
        if transformed[0] != 2 {
            let error = 1 / 0
        }

        if transformed[1] != 5 {
            let error = 1 / 0
        }

        if transformed[4] != 26 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_list_comprehension_with_complex_filter() {
    let source = r#"
        let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        let special = [x * 2 for x in numbers if x > 5]

        # Should be: 6*2=12, 7*2=14, 8*2=16, 9*2=18, 10*2=20
        if special.len() != 5 {
            let error = 1 / 0
        }

        if special[0] != 12 {
            let error = 1 / 0
        }

        if special[4] != 20 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_list_comprehension_empty_result() {
    let source = r#"
        let numbers = [1, 2, 3, 4, 5]
        let filtered = [x for x in numbers if x > 10]

        if filtered.len() != 0 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_list_comprehension_nested_access() {
    let source = r#"
        let lists = [[1, 2], [3, 4], [5, 6]]
        let firsts = [list[0] for list in lists]

        if firsts.len() != 3 {
            let error = 1 / 0
        }

        if firsts[0] != 1 {
            let error = 1 / 0
        }

        if firsts[2] != 5 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_list_comprehension_with_range_emulation() {
    let source = r#"
        # Emulate range using explicit list
        let range = [0, 1, 2, 3, 4]
        let squares = [x * x for x in range]

        if squares[0] != 0 {
            let error = 1 / 0
        }

        if squares[2] != 4 {
            let error = 1 / 0
        }

        if squares[4] != 16 {
            let error = 1 / 0
        }
    "#;

    run_test(source);
}

#[test]
fn test_list_comprehension_string_operations() {
    let source = r#"
        let names = ["alice", "bob", "charlie"]
        let lengths = [name.len() for name in names]

        if lengths[0] != 5 {  # alice = 5
            let error = 1 / 0
        }

        if lengths[1] != 3 {  # bob = 3
            let error = 1 / 0
        }

        if lengths[2] != 7 {  # charlie = 7
            let error = 1 / 0
        }
    "#;

    run_test(source);
}
