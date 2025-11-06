mod helpers;
use helpers::run_test;

// += tests
#[test]
fn test_add_assign_numbers() {
    run_test(r#"
        mut x = 10
        x += 5
        if x != 15 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_add_assign_chain() {
    run_test(r#"
        mut x = 1
        x += 2
        x += 3
        x += 4
        if x != 10 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_add_assign_negative() {
    run_test(r#"
        mut x = 10
        x += -5
        if x != 5 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_add_assign_zero() {
    run_test(r#"
        mut x = 10
        x += 0
        if x != 10 {
            let error = 1 / 0
        }
    "#);
}

// -= tests
#[test]
fn test_sub_assign_numbers() {
    run_test(r#"
        mut x = 10
        x -= 3
        if x != 7 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_sub_assign_chain() {
    run_test(r#"
        mut x = 20
        x -= 5
        x -= 3
        x -= 2
        if x != 10 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_sub_assign_negative() {
    run_test(r#"
        mut x = 10
        x -= -5
        if x != 15 {
            let error = 1 / 0
        }
    "#);
}

// *= tests
#[test]
fn test_mul_assign_numbers() {
    run_test(r#"
        mut x = 5
        x *= 3
        if x != 15 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_mul_assign_chain() {
    run_test(r#"
        mut x = 2
        x *= 3
        x *= 4
        if x != 24 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_mul_assign_zero() {
    run_test(r#"
        mut x = 10
        x *= 0
        if x != 0 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_mul_assign_one() {
    run_test(r#"
        mut x = 42
        x *= 1
        if x != 42 {
            let error = 1 / 0
        }
    "#);
}

// /= tests
#[test]
fn test_div_assign_numbers() {
    run_test(r#"
        mut x = 20
        x /= 4
        if x != 5 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_div_assign_chain() {
    run_test(r#"
        mut x = 100
        x /= 2
        x /= 5
        if x != 10 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_div_assign_one() {
    run_test(r#"
        mut x = 42
        x /= 1
        if x != 42 {
            let error = 1 / 0
        }
    "#);
}

// %= tests
#[test]
fn test_mod_assign_numbers() {
    run_test(r#"
        mut x = 17
        x %= 5
        if x != 2 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_mod_assign_chain() {
    run_test(r#"
        mut x = 20
        x %= 7
        x %= 4
        if x != 2 {
            let error = 1 / 0
        }
    "#);
}

// Combined operators
#[test]
fn test_combined_assign_operators() {
    run_test(r#"
        mut x = 10
        x += 5   # 15
        x *= 2   # 30
        x -= 10  # 20
        x /= 4   # 5
        x %= 3   # 2
        if x != 2 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_assign_with_expressions() {
    run_test(r#"
        mut x = 10
        let y = 5
        x += y * 2
        if x != 20 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_assign_in_loop() {
    run_test(r#"
        mut sum = 0
        for i in [1, 2, 3, 4, 5] {
            sum += i
        }
        if sum != 15 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_assign_with_function_call() {
    run_test(r#"
        fn get_value() {
            return 7
        }

        mut x = 10
        x += get_value()
        if x != 17 {
            let error = 1 / 0
        }
    "#);
}
