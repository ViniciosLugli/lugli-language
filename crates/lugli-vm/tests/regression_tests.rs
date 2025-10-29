// Regression tests for issues fixed during hangman.lg implementation
mod helpers;
use helpers::run_test;

#[test]
fn test_mutating_method_bang_suffix() {
    // Regression test: Method names with ! suffix were not being parsed
    let source = r#"
        struct Container {
            items: []

            fn add!(self, item) {
                self.items.push!(item)
            }

            fn clear!(self) {
                self.items = []
            }
        }

        let c = Container { items: [] }
        c.add!("test")

        if c.items.len() != 1 {
            let error = 1 / 0  # Mutating method with ! failed
        }

        c.clear!()
        if c.items.len() != 0 {
            let error = 1 / 0  # Clear! method failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_self_parameter_in_methods() {
    // Regression test: 'self' was not recognized as a valid identifier
    let source = r#"
        struct Calculator {
            value: 0

            fn add(self, n) {
                return self.value + n
            }

            fn set!(self, n) {
                self.value = n
            }

            fn get(self) {
                return self.value
            }
        }

        let calc = Calculator { value: 10 }

        if calc.add(5) != 15 {
            let error = 1 / 0  # Method with self parameter failed
        }

        calc.set!(20)
        if calc.get() != 20 {
            let error = 1 / 0  # Self parameter access failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_pascalcase_struct_literal_disambiguation() {
    // Regression test: PascalCase struct literals were being parsed as method calls
    let source = r#"
        struct Person {
            name: ""
            age: 0
        }

        struct ADDRESS {
            street: ""
        }

        # PascalCase should be recognized as struct literal
        let p = Person { name: "Alice", age: 30 }

        # ALL_CAPS should also work
        let a = ADDRESS { street: "Main St" }

        if p.name != "Alice" {
            let error = 1 / 0  # PascalCase struct literal failed
        }

        if a.street != "Main St" {
            let error = 1 / 0  # ALL_CAPS struct literal failed
        }

        # lowercase should be method call (would fail if no such method)
        struct Helper {
            fn helper() {
                return 42
            }
        }

        # This would be a method call, not struct literal
        # let h = helper()  // This would look for a function
    "#;

    run_test(source);
}

#[test]
fn test_let_mut_declaration() {
    // Regression test: 'let mut' was not parsing correctly
    let source = r#"
        let mut counter = 0
        counter = counter + 1

        if counter != 1 {
            let error = 1 / 0  # let mut failed
        }

        # Regular let should also work
        let value = 42
        if value != 42 {
            let error = 1 / 0  # Regular let failed
        }

        # Test mutation
        let mut list = []
        list.push!(1)
        list.push!(2)

        if list.len() != 2 {
            let error = 1 / 0  # Mutable list operations failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_loop_statement() {
    // Regression test: 'loop' statement was not implemented
    let source = r#"
        let counter = 0

        loop {
            counter = counter + 1
            if counter >= 5 {
                break
            }
        }

        if counter != 5 {
            let error = 1 / 0  # Loop with break failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_for_loop_with_range() {
    // Regression test: 'for' loops were not implemented
    let source = r#"
        let sum = 0

        for i in range(5) {
            sum = sum + i
        }

        if sum != 10 {  # 0 + 1 + 2 + 3 + 4
            let error = 1 / 0  # For loop with range failed
        }

        # For loop with list
        let product = 1
        for n in [2, 3, 4] {
            product = product * n
        }

        if product != 24 {
            let error = 1 / 0  # For loop with list failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_break_continue_in_loops() {
    // Regression test: break/continue were not properly implemented
    let source = r#"
        let sum = 0
        let iterations = 0

        for i in range(10) {
            iterations = iterations + 1

            if i == 2 {
                continue  # Skip 2
            }

            if i == 5 {
                break  # Stop at 5
            }

            sum = sum + i
        }

        # Should have: 0 + 1 + 3 + 4 = 8
        if sum != 8 {
            let error = 1 / 0  # Continue/break logic failed
        }

        # Should have iterated 6 times (0,1,2,3,4,5)
        if iterations != 6 {
            let error = 1 / 0  # Break didn't stop loop correctly
        }
    "#;

    run_test(source);
}

#[test]
fn test_no_stack_pollution_in_loops() {
    // Regression test: Expression statements in loops were polluting the stack
    let source = r#"
        let list = []

        for i in range(3) {
            list.push!(i)  # This was leaving values on stack
            i * 2  # Expression statement - should not pollute stack
            "test"  # Another expression
        }

        # Stack should be clean after loop
        let result = 42

        if result != 42 {
            let error = 1 / 0  # Stack polluted after loop
        }

        if list.len() != 3 {
            let error = 1 / 0  # Loop operations failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_method_return_values_in_loops() {
    // Regression test: Methods in loops were returning null due to stack issues
    let source = r#"
        let words = ["hello", "world"]
        let uppercase = []

        for word in words {
            let upper = word.upper()  # This was returning null
            uppercase.push!(upper)
        }

        if uppercase[0] != "HELLO" {
            let error = 1 / 0  # Method return in loop failed
        }

        if uppercase[1] != "WORLD" {
            let error = 1 / 0  # Second method return in loop failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_jumpiffalse_consumes_condition() {
    // Regression test: JumpIfFalse was not consuming the condition value
    let source = r#"
        fn check_and_return(value) {
            if value {
                return "true"
            }
            return "false"
        }

        let result1 = check_and_return(true)
        let result2 = check_and_return(false)

        if result1 != "true" {
            let error = 1 / 0  # JumpIfFalse with true condition failed
        }

        if result2 != "false" {
            let error = 1 / 0  # JumpIfFalse with false condition failed
        }

        # Complex condition
        let x = 5
        if x > 3 && x < 10 {
            # Should reach here
        } else {
            let error = 1 / 0  # Complex condition evaluation failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_input_chain_in_loop() {
    // Regression test: Input chains were returning wrong values
    // This simulates the hangman input scenario
    let source = r#"
        struct MockInput {
            values: []
            index: 0

            fn get_next(self) {
                if self.index < self.values.len() {
                    let value = self.values[self.index]
                    self.index = self.index + 1
                    return value
                }
                return ""
            }
        }

        let input = MockInput { values: ["a", "b", "c"], index: 0 }
        let results = []

        while input.index < input.values.len() {
            let value = input.get_next()
            results.push!(value)
        }

        if results.len() != 3 {
            let error = 1 / 0  # Input chain failed - wrong length
        }

        if results[0] != "a" || results[1] != "b" || results[2] != "c" {
            let error = 1 / 0  # Input chain returned wrong values
        }
    "#;

    run_test(source);
}

#[test]
fn test_nested_loops_with_break() {
    // Regression test: Nested loops with break were not working correctly
    let source = r#"
        let result = []

        for i in range(3) {
            for j in range(3) {
                if i == 1 && j == 1 {
                    break  # Should only break inner loop
                }
                result.push!(i * 10 + j)
            }
        }

        # Should have: 0,1,2,10,20,21,22
        if result.len() != 7 {
            let error = 1 / 0  # Nested loop break failed
        }

        if result[3] != 10 {
            let error = 1 / 0  # Nested loop resume after break failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_string_methods_in_struct() {
    // Regression test: String methods on struct fields
    let source = r#"
        struct Message {
            text: ""

            fn shout(self) {
                return self.text.upper() + "!"
            }

            fn whisper(self) {
                return self.text.lower()
            }
        }

        let msg = Message { text: "Hello World" }

        if msg.shout() != "HELLO WORLD!" {
            let error = 1 / 0  # String method on struct field failed
        }

        if msg.whisper() != "hello world" {
            let error = 1 / 0  # Lower case method failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_dict_get_with_default() {
    // Regression test: Dict get with default value
    let source = r#"
        let config = {
            "host": "localhost",
            "port": 8080
        }

        let host = config.get("host", "default")
        let timeout = config.get("timeout", 30)

        if host != "localhost" {
            let error = 1 / 0  # Dict get existing key failed
        }

        if timeout != 30 {
            let error = 1 / 0  # Dict get with default failed
        }
    "#;

    run_test(source);
}

#[test]
fn test_list_operations_sequence() {
    // Regression test: Complex sequence of list operations
    let source = r#"
        let list = []

        # Build list
        for i in range(5) {
            list.push!(i)
        }

        # Filter evens
        let evens = list.filter(fn(x) { x % 2 == 0 })

        # Map to double
        let doubled = evens.map!(fn(x) { x * 2 })

        if doubled.len() != 3 {  # 0, 2, 4
            let error = 1 / 0  # Filter + map sequence failed
        }

        if doubled[0] != 0 || doubled[1] != 4 || doubled[2] != 8 {
            let error = 1 / 0  # Filter + map values wrong
        }

        # Original list should be unchanged
        if list.len() != 5 {
            let error = 1 / 0  # Original list was modified
        }
    "#;

    run_test(source);
}
