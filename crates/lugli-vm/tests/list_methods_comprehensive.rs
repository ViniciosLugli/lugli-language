mod helpers;
use helpers::run_test;

// push() tests
#[test]
fn test_list_push_basic() {
    run_test(r#"
        let list = []
        list.push(1)
        list.push(2)
        list.push(3)
        if list.len() != 3 {
            let error = 1 / 0
        }
        if list[2] != 3 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_push_different_types() {
    run_test(r#"
        let list = []
        list.push(1)
        list.push("hello")
        list.push(true)
        if list.len() != 3 {
            let error = 1 / 0
        }
    "#);
}

// pop() tests
#[test]
fn test_list_pop_basic() {
    run_test(r#"
        let list = [1, 2, 3]
        let val = list.pop()
        if val != 3 {
            let error = 1 / 0
        }
        if list.len() != 2 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_pop_empty() {
    run_test(r#"
        let list = []
        let val = list.pop()
        if val != null {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_pop_until_empty() {
    run_test(r#"
        let list = [1, 2]
        list.pop()
        list.pop()
        if list.len() != 0 {
            let error = 1 / 0
        }
        let val = list.pop()
        if val != null {
            let error = 1 / 0
        }
    "#);
}

// get() tests (with negative index support)
#[test]
fn test_list_get_valid_index() {
    run_test(r#"
        let list = [10, 20, 30]
        if list.get(0) != 10 {
            let error = 1 / 0
        }
        if list.get(2) != 30 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_get_out_of_bounds() {
    run_test(r#"
        let list = [1, 2, 3]
        if list.get(10) != null {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_get_negative_index() {
    run_test(r#"
        let list = [10, 20, 30]
        if list.get(-1) != 30 {
            let error = 1 / 0
        }
        if list.get(-3) != 10 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_get_negative_out_of_bounds() {
    run_test(r#"
        let list = [1, 2, 3]
        if list.get(-10) != null {
            let error = 1 / 0
        }
    "#);
}

// set() tests (with negative index support)
#[test]
fn test_list_set_valid_index() {
    run_test(r#"
        let list = [10, 20, 30]
        list.set(1, 99)
        if list[1] != 99 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_set_negative_index() {
    run_test(r#"
        let list = [10, 20, 30]
        list.set(-1, 99)
        if list[2] != 99 {
            let error = 1 / 0
        }
        list.set(-3, 11)
        if list[0] != 11 {
            let error = 1 / 0
        }
    "#);
}

// len() tests
#[test]
fn test_list_len_basic() {
    run_test(r#"
        let list = [1, 2, 3, 4, 5]
        if list.len() != 5 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_len_empty() {
    run_test(r#"
        let list = []
        if list.len() != 0 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_len_after_push() {
    run_test(r#"
        let list = [1, 2]
        list.push(3)
        if list.len() != 3 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_len_after_pop() {
    run_test(r#"
        let list = [1, 2, 3]
        list.pop()
        if list.len() != 2 {
            let error = 1 / 0
        }
    "#);
}

// contains() tests
#[test]
fn test_list_contains_found() {
    run_test(r#"
        let list = [1, 2, 3, 4, 5]
        if !list.contains(3) {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_contains_not_found() {
    run_test(r#"
        let list = [1, 2, 3]
        if list.contains(10) {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_contains_empty_list() {
    run_test(r#"
        let list = []
        if list.contains(1) {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_contains_different_types() {
    run_test(r#"
        let list = [1, "hello", true]
        if !list.contains("hello") {
            let error = 1 / 0
        }
        if !list.contains(true) {
            let error = 1 / 0
        }
    "#);
}

// reverse() tests
#[test]
fn test_list_reverse_basic() {
    run_test(r#"
        let list = [1, 2, 3, 4, 5]
        list.reverse()
        if list[0] != 5 || list[4] != 1 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_reverse_empty() {
    run_test(r#"
        let list = []
        list.reverse()
        if list.len() != 0 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_reverse_single_element() {
    run_test(r#"
        let list = [42]
        list.reverse()
        if list[0] != 42 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_reverse_two_elements() {
    run_test(r#"
        let list = [1, 2]
        list.reverse()
        if list[0] != 2 || list[1] != 1 {
            let error = 1 / 0
        }
    "#);
}

// clear() tests
#[test]
fn test_list_clear_basic() {
    run_test(r#"
        let list = [1, 2, 3, 4, 5]
        list.clear()
        if list.len() != 0 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_clear_already_empty() {
    run_test(r#"
        let list = []
        list.clear()
        if list.len() != 0 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_clear_then_add() {
    run_test(r#"
        let list = [1, 2, 3]
        list.clear()
        list.push(10)
        if list.len() != 1 || list[0] != 10 {
            let error = 1 / 0
        }
    "#);
}

// is_empty() tests
#[test]
fn test_list_is_empty_true() {
    run_test(r#"
        let list = []
        if !list.is_empty() {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_is_empty_false() {
    run_test(r#"
        let list = [1]
        if list.is_empty() {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_is_empty_after_clear() {
    run_test(r#"
        let list = [1, 2, 3]
        list.clear()
        if !list.is_empty() {
            let error = 1 / 0
        }
    "#);
}

// append() tests
#[test]
fn test_list_append_basic() {
    run_test(r#"
        let list1 = [1, 2]
        let list2 = [3, 4]
        list1.append(list2)
        if list1.len() != 4 {
            let error = 1 / 0
        }
        if list1[3] != 4 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_append_empty_to_list() {
    run_test(r#"
        let list1 = [1, 2]
        let list2 = []
        list1.append(list2)
        if list1.len() != 2 {
            let error = 1 / 0
        }
    "#);
}

#[test]
fn test_list_append_to_empty() {
    run_test(r#"
        let list1 = []
        let list2 = [1, 2, 3]
        list1.append(list2)
        if list1.len() != 3 {
            let error = 1 / 0
        }
    "#);
}

// Combined operations
#[test]
fn test_list_combined_operations() {
    run_test(r#"
        let list = []
        list.push(1)
        list.push(2)
        list.push(3)
        list.reverse()
        let val = list.pop()
        if val != 1 {
            let error = 1 / 0
        }
        if list.len() != 2 {
            let error = 1 / 0
        }
    "#);
}
