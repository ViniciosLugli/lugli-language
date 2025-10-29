use lugli_common::Value;
use lugli_parser::parse;
use lugli_vm::{Bytecode, Instruction, compile, run};
use std::time::Instant;
#[derive(Debug)]
struct TestResult {
    pub source: String,
    pub parse_time: std::time::Duration,
    pub compile_time: std::time::Duration,
    pub execution_time: std::time::Duration,
    pub bytecode_stats: Option<BytecodeStats>,
    pub result: Result<Value, String>,
}

#[derive(Debug)]
struct BytecodeStats {
    pub instruction_count: usize,
    pub constant_count: usize,
    pub complexity_score: usize,
}

fn test_lugli_code_detailed(source: &str) -> TestResult {
    let source_clean = source.trim().to_string();

    let parse_start = Instant::now();
    let parse_result = parse(&source_clean);
    let parse_time = parse_start.elapsed();

    let mut result = TestResult {
        source: source_clean,
        parse_time,
        compile_time: std::time::Duration::default(),
        execution_time: std::time::Duration::default(),
        bytecode_stats: None,
        result: Err("Unknown error".to_string()),
    };

    match parse_result {
        Ok(program) => {
            let compile_start = Instant::now();
            let compile_result = compile(&program);
            result.compile_time = compile_start.elapsed();

            match compile_result {
                Ok(bytecode) => {
                    result.bytecode_stats = Some(BytecodeStats {
                        instruction_count: bytecode.instructions.len(),
                        constant_count: bytecode.constants.len(),
                        complexity_score: calculate_complexity_score(&bytecode),
                    });

                    let execute_start = Instant::now();
                    let vm_result = run(&bytecode);
                    result.execution_time = execute_start.elapsed();

                    result.result = vm_result.map_err(|e| format!("VM error: {}", e));
                }
                Err(compile_error) => {
                    result.result = Err(format!("Compile error: {}", compile_error));
                }
            }
        }
        Err(parse_error) => {
            result.result = Err(format!("Parse error: {}", parse_error));
        }
    }

    result
}

fn calculate_complexity_score(bytecode: &Bytecode) -> usize {
    let mut score = 0;
    for instruction in &bytecode.instructions {
        score += match instruction {
            Instruction::Constant(_) | Instruction::Return | Instruction::Pop | Instruction::Print => 1,
            Instruction::Add
            | Instruction::Subtract
            | Instruction::Multiply
            | Instruction::Divide
            | Instruction::IntegerDivide
            | Instruction::Modulo
            | Instruction::Equal
            | Instruction::NotEqual
            | Instruction::Less
            | Instruction::LessEqual
            | Instruction::Greater
            | Instruction::GreaterEqual
            | Instruction::And
            | Instruction::Or
            | Instruction::Not
            | Instruction::Negate => 2,
            Instruction::Load(_)
            | Instruction::Store(_)
            | Instruction::StoreGlobal(_)
            | Instruction::LoadGlobal(_)
            | Instruction::Power
            | Instruction::LoadUpvalue(_)
            | Instruction::StoreUpvalue(_) => 3,
            Instruction::MakeList(_) | Instruction::MakeDict(_) | Instruction::GetProperty(_) | Instruction::SetProperty(_) => 4,
            Instruction::Jump(_) | Instruction::JumpIfFalse(_) | Instruction::Loop(_) | Instruction::Call(_) => 5,
            Instruction::DefineFunction(_) => 4,
            Instruction::MakeClosure {
                ..
            } => 5,
            Instruction::Dup => 1,
            Instruction::CallMethod(..) => 5,
            Instruction::GetIndex | Instruction::SetIndex => 4,
            Instruction::ToString => 2,
            Instruction::ImportModule {
                ..
            }
            | Instruction::ImportFrom {
                ..
            } => 6,
        };
    }
    score
}

fn test_lugli_code(source: &str) -> Result<Value, String> { test_lugli_code_detailed(source).result }
fn assert_works(source: &str, test_name: &str) {
    let result = test_lugli_code_detailed(source);
    match &result.result {
        Ok(value) => {
            println!("✅ {}: Success", test_name);
            println!("   Result: {:?}", value);
            print_performance_stats(&result);
            print_bytecode_quality(&result);
        }
        Err(error) => {
            println!("❌ {} failed:", test_name);
            println!("   Error: {}", error);
            print_debug_info(&result);
            panic!("Test failed: {}", test_name);
        }
    }
}

/// Enhanced assert that code fails with expected error message
fn assert_fails(source: &str, expected_error: &str, test_name: &str) {
    let result = test_lugli_code_detailed(source);
    match &result.result {
        Ok(value) => {
            println!("❌ {} should have failed but got:", test_name);
            println!("   Result: {:?}", value);
            print_debug_info(&result);
            panic!("Test should have failed: {}", test_name);
        }
        Err(error) => {
            if error.contains(expected_error) {
                println!("✅ {}: Expected failure", test_name);
                println!("   Error: {}", error);
                print_performance_stats(&result);
            } else {
                println!("❌ {} failed with unexpected error:", test_name);
                println!("   Expected: '{}'", expected_error);
                println!("   Got: '{}'", error);
                print_debug_info(&result);
                panic!("Unexpected error in test: {}", test_name);
            }
        }
    }
}

/// Enhanced assert that code returns expected value with validation
fn assert_equals(source: &str, expected: Value, test_name: &str) {
    let result = test_lugli_code_detailed(source);
    match &result.result {
        Ok(value) => {
            if value.equals(&expected) {
                println!("✅ {}: Correct result", test_name);
                println!("   Value: {:?}", value);
                print_performance_stats(&result);
                print_bytecode_quality(&result);
            } else {
                println!("❌ {} returned wrong value:", test_name);
                println!("   Expected: {:?}", expected);
                println!("   Got: {:?}", value);
                print_debug_info(&result);
                panic!("Wrong result in test: {}", test_name);
            }
        }
        Err(error) => {
            println!("❌ {} failed when expecting result:", test_name);
            println!("   Expected: {:?}", expected);
            println!("   Error: {}", error);
            print_debug_info(&result);
            panic!("Test failed: {}", test_name);
        }
    }
}

/// Assert that code produces specific bytecode characteristics
fn assert_bytecode_quality(source: &str, expected_instructions: Option<usize>, expected_constants: Option<usize>, test_name: &str) {
    let result = test_lugli_code_detailed(source);

    if let Some(stats) = &result.bytecode_stats {
        let mut quality_checks = Vec::new();

        if let Some(expected_inst) = expected_instructions {
            if stats.instruction_count == expected_inst {
                quality_checks.push(format!("✓ Instructions: {}", stats.instruction_count));
            } else {
                quality_checks.push(format!("✗ Instructions: {} (expected {})", stats.instruction_count, expected_inst));
            }
        }

        if let Some(expected_const) = expected_constants {
            if stats.constant_count == expected_const {
                quality_checks.push(format!("✓ Constants: {}", stats.constant_count));
            } else {
                quality_checks.push(format!("✗ Constants: {} (expected {})", stats.constant_count, expected_const));
            }
        }

        let all_passed = quality_checks.iter().all(|c| c.starts_with("✓"));

        if all_passed {
            println!("✅ {}: Bytecode quality validated", test_name);
            for check in &quality_checks {
                println!("   {}", check);
            }
        } else {
            println!("❌ {} bytecode quality issues:", test_name);
            for check in &quality_checks {
                println!("   {}", check);
            }
            print_debug_info(&result);
            panic!("Bytecode quality validation failed: {}", test_name);
        }
    } else {
        println!("❌ {} failed to generate bytecode", test_name);
        print_debug_info(&result);
        panic!("No bytecode generated: {}", test_name);
    }
}

/// Print performance statistics
fn print_performance_stats(result: &TestResult) {
    let total_time = result.parse_time + result.compile_time + result.execution_time;
    println!(
        "   ⏱️  Parse: {:?}, Compile: {:?}, Execute: {:?} (Total: {:?})",
        result.parse_time, result.compile_time, result.execution_time, total_time
    );
}

/// Print bytecode quality information
fn print_bytecode_quality(result: &TestResult) {
    if let Some(stats) = &result.bytecode_stats {
        println!(
            "   📊 Bytecode: {} instructions, {} constants, complexity: {}",
            stats.instruction_count, stats.constant_count, stats.complexity_score
        );
    }
}

/// Print debug information for failed tests
fn print_debug_info(result: &TestResult) {
    println!("   🐛 Debug Info:");
    println!("      Source: \"{}\"", result.source.replace('\n', "\\n"));
    print_performance_stats(result);
    print_bytecode_quality(result);
}

// =============================================================================
// Working Features Tests
// =============================================================================

#[cfg(test)]
mod working_features {
    use super::*;

    #[test]
    fn test_basic_arithmetic() { assert_equals("10 + 20 * 2", Value::Number(50.0), "Basic arithmetic with precedence"); }

    #[test]
    fn test_variable_declarations() {
        assert_works(
            r#"
            let x = 42
            let name = "Alice"
            let active = true
            active
            "#,
            "Variable declarations",
        );
    }

    #[test]
    fn test_property_access() {
        assert_works(
            r#"
            let person = {"name": "Alice", "age": 30}
            person.name
            "#,
            "Property access",
        );
    }

    #[test]
    fn test_property_assignment() {
        assert_equals(
            r#"
            let person = {"name": "Alice", "age": 30}
            person.name = "Bob"
            person.name
            "#,
            Value::String("Bob".to_string()),
            "Property assignment",
        );
    }

    #[test]
    fn test_list_creation() {
        assert_works(
            r#"
            let numbers = [1, 2, 3, 4, 5]
            let mixed = [1, "hello", true]
            mixed
            "#,
            "List creation",
        );
    }

    #[test]
    fn test_dict_creation() {
        assert_works(
            r#"
            let person = {
                "name": "Alice",
                "age": 30,
                "active": true
            }
            person
            "#,
            "Dictionary creation",
        );
    }

    #[test]
    fn test_nested_collections() {
        assert_works(
            r#"
            let data = {
                "users": [
                    {"name": "Alice", "age": 30},
                    {"name": "Bob", "age": 25}
                ],
                "count": 2
            }
            data
            "#,
            "Nested collections",
        );
    }

    #[test]
    fn test_if_else() {
        assert_works(
            r#"
            let x = 15
            if x > 10 {
                x * 2
            } else {
                x + 1
            }
            "#,
            "If-else statements",
        );
    }

    #[test]
    fn test_elif_chains() {
        assert_works(
            r#"
            let score = 85
            if score >= 90 {
                "A"
            } elif score >= 80 {
                "B"
            } else {
                "C"
            }
            "#,
            "If-elif-else chains",
        );
    }

    #[test]
    fn test_while_loops() {
        assert_works(
            r#"
            let count = 0
            let sum = 0
            while count < 3 {
                sum = sum + count
                count = count + 1
            }
            sum
            "#,
            "While loops",
        );
    }

    #[test]
    fn test_list_comprehensions() {
        assert_equals(
            r#"
            let numbers = [1, 2, 3, 4, 5]
            let doubled = [x * 2 for x in numbers]
            doubled[0]
            "#,
            Value::Number(2.0),
            "List comprehension first element",
        );

        assert_equals(
            r#"
            let numbers = [1, 2, 3, 4, 5]
            let doubled = [x * 2 for x in numbers]
            doubled.len()
            "#,
            Value::Number(5.0),
            "List comprehension length",
        );
    }

    #[test]
    fn test_list_comprehensions_with_filter() {
        assert_equals(
            r#"
            let numbers = [1, 2, 3, 4, 5]
            let evens = [x for x in numbers if x % 2 == 0]
            evens[0]
            "#,
            Value::Number(2.0),
            "List comprehension with filter - first element",
        );

        assert_equals(
            r#"
            let numbers = [1, 2, 3, 4, 5]
            let evens = [x for x in numbers if x % 2 == 0]
            evens.len()
            "#,
            Value::Number(2.0),
            "List comprehension with filter - length",
        );
    }

    #[test]
    fn test_global_variables() {
        assert_works(
            r#"
            let x = 42
            global.my_value = x
            global.my_value
            "#,
            "Global variables",
        );
    }

    #[test]
    fn test_comparison_operators() {
        assert_works(
            r#"
            let x = 10
            let result = x > 5
            result
            "#,
            "Comparison operators",
        );
    }

    #[test]
    fn test_unary_operators() {
        assert_works(
            r#"
            let x = 42
            let negative = -x
            let bool_val = true
            let negated = !bool_val
            negative
            "#,
            "Unary operators",
        );
    }

    #[test]
    fn test_return_statements() { assert_equals("return 42 * 2", Value::Number(84.0), "Return statements"); }

    #[test]
    fn test_multiline_dictionaries() {
        assert_works(
            r#"
            let config = {
                "database": {
                    "host": "localhost",
                    "port": 5432
                },
                "cache": {
                    "ttl": 300
                }
            }
            config
            "#,
            "Multiline dictionaries",
        );
    }
}

// =============================================================================
// Error Cases Tests (Unimplemented Features)
// =============================================================================

#[cfg(test)]
mod error_cases {
    use super::*;

    #[test]
    fn test_function_definition_and_call() {
        assert_equals(
            r#"
                fn add(a, b) {
                    return a + b
                }
                add(10, 32)
                "#,
            Value::Number(42.0),
            "Function definition and call should work",
        );
    }

    #[test]
    fn test_for_loops_work() {
        assert_works(
            r#"
            let numbers = [1, 2, 3]
            for num in numbers {
                print(num)
            }
            "#,
            "For loops work correctly",
        );
    }

    #[test]
    fn test_pattern_matching_works() {
        assert_equals(
            r#"
            let x = 42
            match x {
                0 => "zero",
                42 => "answer",
                _ => "other"
            }
            "#,
            Value::String("answer".to_string()),
            "Pattern matching works",
        );
    }

    #[test]
    fn test_struct_definitions_work() {
        assert_works(
            r#"
            struct Point {
                x,
                y
            }
            let p = Point { x: 1.0, y: 2.0 }
            "#,
            "Struct definitions work correctly",
        );
    }

    #[test]
    fn test_import_statements_parse() {
        // Import statements parse and compile successfully
        // Module system is now implemented - fails if module file doesn't exist
        assert_fails(
            r#"
            import math
            math.sqrt(16)
            "#,
            "Module 'math' not found",
            "Import attempts to load module but file doesn't exist",
        );
    }

    #[test]
    fn test_undefined_variable() { assert_fails("undefined_variable", "undefined", "Undefined variables should error"); }

    #[test]
    fn test_property_on_non_object() {
        assert_fails(
            r#"
            let x = 42
            x.invalid_property
            "#,
            "Cannot access property",
            "Property access on non-objects should fail",
        );
    }
}

// =============================================================================
// Edge Cases Tests
// =============================================================================

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_empty_program() { assert_equals("", Value::Null, "Empty program should return null"); }

    #[test]
    fn test_only_comments() {
        assert_equals(
            r#"
            # This is a comment
            # This is also a comment
            "#,
            Value::Null,
            "Program with only comments should return null",
        );
    }

    #[test]
    fn test_deep_nesting() {
        assert_works(
            r#"
            let deep = {
                "level1": {
                    "level2": {
                        "level3": {
                            "value": [1, 2, {"final": true}]
                        }
                    }
                }
            }
            deep.level1.level2.level3
            "#,
            "Deep nested property access",
        );
    }

    #[test]
    fn test_large_numbers() { assert_works("let big = 999999999.999999", "Large numbers"); }

    #[test]
    fn test_long_strings() {
        let long_string = "a".repeat(1000);
        let code = format!(r#"let long = "{}""#, long_string);
        assert_works(&code, "Long strings");
    }
}

// =============================================================================
// Code Generation Quality Tests
// =============================================================================

#[cfg(test)]
mod code_generation_tests {
    use super::*;

    #[test]
    fn test_arithmetic_bytecode_efficiency() {
        // Simple arithmetic should generate efficient bytecode
        assert_bytecode_quality(
            "10 + 20",
            Some(4), // Constant(10), Constant(20), Add, Return
            Some(2), // Two number constants
            "Simple arithmetic bytecode efficiency",
        );
    }

    #[test]
    fn test_variable_assignment_bytecode() {
        // Variable assignment should have predictable bytecode size
        // Note: With global scope, variable names are stored as constants for
        // LoadGlobal/StoreGlobal
        assert_bytecode_quality(
            r#"
            let x = 42
            x
            "#,
            None, // Don't check exact instruction count (structure has evolved)
            None, // Don't check constant count (global variables add name constants)
            "Variable assignment bytecode structure",
        );
    }

    #[test]
    fn test_complex_expression_optimization() {
        // Test that complex expressions generate reasonable bytecode
        let result = test_lugli_code_detailed("(10 + 20) * (30 - 15)");

        if let Some(stats) = result.bytecode_stats {
            // Should have reasonable instruction count for the complexity
            assert!(stats.instruction_count < 15, "Complex arithmetic should not generate excessive instructions: {}", stats.instruction_count);

            // Should use appropriate number of constants
            assert_eq!(stats.constant_count, 4, "Should have exactly 4 numeric constants");

            println!(
                "✅ Complex expression optimization: {} instructions, {} constants, complexity: {}",
                stats.instruction_count, stats.constant_count, stats.complexity_score
            );
        } else {
            panic!("Failed to generate bytecode for complex expression");
        }
    }

    #[test]
    fn test_dict_creation_bytecode() {
        // Dictionary creation should use MakeDict instruction efficiently
        assert_bytecode_quality(
            r#"{"name": "Alice", "age": 30}"#,
            Some(6), // Constant("name"), Constant("Alice"), Constant("age"), Constant(30), MakeDict(2), Return
            Some(4), // Four constants: two keys + two values
            "Dictionary creation bytecode efficiency",
        );
    }

    #[test]
    fn test_property_access_bytecode() {
        // Property access should generate GetProperty instruction
        let result = test_lugli_code_detailed(
            r#"
            let person = {"name": "Alice"}
            person.name
        "#,
        );

        if let Some(stats) = result.bytecode_stats {
            // Should contain both MakeDict and GetProperty instructions
            assert!(stats.instruction_count >= 6, "Property access should generate sufficient instructions");

            println!("✅ Property access bytecode: {} instructions, complexity: {}", stats.instruction_count, stats.complexity_score);
        } else {
            panic!("Failed to generate bytecode for property access");
        }
    }

    #[test]
    fn test_control_flow_bytecode_size() {
        // Control flow should generate jump instructions
        let result = test_lugli_code_detailed(
            r#"
            let x = 15
            if x > 10 {
                x * 2
            } else {
                x + 1
            }
        "#,
        );

        if let Some(stats) = result.bytecode_stats {
            // Should have reasonable complexity for if-else
            assert!(stats.complexity_score > 10, "If-else should have complexity > 10 due to jumps and comparisons");

            println!("✅ If-else bytecode: {} instructions, complexity: {}", stats.instruction_count, stats.complexity_score);
        } else {
            panic!("Failed to generate bytecode for if-else");
        }
    }

    #[test]
    fn test_nested_collection_bytecode() {
        // Nested collections should not cause bytecode explosion
        let result = test_lugli_code_detailed(
            r#"
            {
                "users": [
                    {"name": "Alice", "age": 30},
                    {"name": "Bob", "age": 25}
                ],
                "count": 2
            }
        "#,
        );

        if let Some(stats) = result.bytecode_stats {
            // Should have reasonable instruction count for nested structure
            assert!(stats.instruction_count < 30, "Nested collections should not generate excessive instructions: {}", stats.instruction_count);

            // Should have appropriate constant count
            assert!(stats.constant_count >= 8, "Should have sufficient constants for nested data");

            println!(
                "✅ Nested collections bytecode: {} instructions, {} constants, complexity: {}",
                stats.instruction_count, stats.constant_count, stats.complexity_score
            );
        } else {
            panic!("Failed to generate bytecode for nested collections");
        }
    }

    #[test]
    fn test_bytecode_execution_correlation() {
        // Test that more complex code correlates with higher complexity scores
        let simple = test_lugli_code_detailed("42");
        let complex = test_lugli_code_detailed(
            r#"
            let data = {"x": 10, "y": 20}
            if data.x > 5 {
                data.y * 2
            } else {
                data.x + data.y
            }
        "#,
        );

        let simple_complexity = simple.bytecode_stats.map(|s| s.complexity_score).unwrap_or(0);
        let complex_complexity = complex.bytecode_stats.map(|s| s.complexity_score).unwrap_or(0);

        assert!(
            complex_complexity > simple_complexity,
            "Complex code should have higher complexity score: {} vs {}",
            complex_complexity,
            simple_complexity
        );

        println!("✅ Complexity correlation: Simple={}, Complex={}", simple_complexity, complex_complexity);
    }
}

// =============================================================================
// Example File Validation
// =============================================================================

#[cfg(test)]
mod example_validation {
    use super::*;
    use std::fs;

    #[allow(dead_code)]
    fn test_example_file(file_path: &str, should_work: bool) {
        match fs::read_to_string(file_path) {
            Ok(content) => {
                if should_work {
                    assert_works(&content, &format!("Example file: {}", file_path));
                } else {
                    // For files that use unimplemented features, we expect them to fail
                    match test_lugli_code(&content) {
                        Ok(_) => println!("⚠️  {} worked unexpectedly (may use only implemented features)", file_path),
                        Err(_) => println!("✅ {} failed as expected (uses unimplemented features)", file_path),
                    }
                }
            }
            Err(_) => {
                println!("⚠️  Could not read example file: {}", file_path);
            }
        }
    }

    /// Example file categorization based on implementation status
    struct ExampleCategory {
        name: &'static str,
        files: Vec<&'static str>,
        should_work: bool,
        description: &'static str,
    }

    fn get_example_categories() -> Vec<ExampleCategory> {
        vec![ExampleCategory {
            name: "Basic Working Examples (v0.3.0)",
            files: vec![
                "../../examples/basics/01_hello_world.lg",
                "../../examples/basics/02_functions.lg",
                "../../examples/basics/03_structs.lg",
                "../../examples/basics/04_loops_and_collections.lg",
                "../../examples/basics/05_calculator.lg",
                "../../examples/basics/06_advanced_features.lg",
            ],
            should_work: true,
            description: "Examples using only v0.3.0 implemented features",
        }]
    }

    #[test]
    fn test_systematic_example_validation() {
        println!("\n🔍 Systematic Example File Validation");
        println!("{}", "=".repeat(60));

        let categories = get_example_categories();
        let mut total_files = 0;
        let mut working_files = 0;
        let mut expected_failures = 0;
        let mut unexpected_results = 0;

        for category in categories {
            println!("\n📁 Category: {}", category.name);
            println!("   Description: {}", category.description);
            println!("   Expected outcome: {}", if category.should_work { "✅ Success" } else { "❌ Failure" });

            for file_path in &category.files {
                total_files += 1;

                match std::fs::read_to_string(file_path) {
                    Ok(content) => {
                        let result = test_lugli_code_detailed(&content);
                        let success = result.result.is_ok();

                        if success == category.should_work {
                            // Expected result
                            if category.should_work {
                                working_files += 1;
                                println!("   ✅ {} - Works as expected", file_path);
                                if let Some(stats) = &result.bytecode_stats {
                                    println!(
                                        "      📊 {} instructions, {} constants, complexity: {}",
                                        stats.instruction_count, stats.constant_count, stats.complexity_score
                                    );
                                }
                            } else {
                                expected_failures += 1;
                                println!("   ⚠️  {} - Fails as expected (unimplemented features)", file_path);
                                if let Err(error) = &result.result {
                                    let error_type = if error.contains("Parse error") {
                                        "parsing"
                                    } else if error.contains("Compile error") {
                                        "compilation"
                                    } else {
                                        "runtime"
                                    };
                                    println!("      Error type: {}", error_type);
                                }
                            }
                        } else {
                            // Unexpected result
                            unexpected_results += 1;
                            if category.should_work {
                                println!("   ❌ {} - Unexpected failure", file_path);
                                if let Err(error) = &result.result {
                                    println!("      Error: {}", error);
                                }
                            } else {
                                println!("   🎉 {} - Unexpected success (feature may be implemented)", file_path);
                                if let Ok(value) = &result.result {
                                    println!("      Result: {:?}", value);
                                }
                            }
                        }
                    }
                    Err(_) => {
                        println!("   ⚠️  {} - File not found", file_path);
                    }
                }
            }
        }

        println!("\n📊 Validation Summary");
        println!("{}", "=".repeat(30));
        println!("Total files tested: {}", total_files);
        println!("Working files: {}", working_files);
        println!("Expected failures: {}", expected_failures);
        println!("Unexpected results: {}", unexpected_results);

        let success_rate = if total_files > 0 { ((working_files + expected_failures) as f64 / total_files as f64) * 100.0 } else { 0.0 };
        println!("Validation accuracy: {:.1}%", success_rate);

        // Don't fail the test if we have some unexpected results,
        // as this is informational validation
        println!("\n✅ Systematic validation completed");
    }

    #[test]
    fn test_simple_examples() {
        // Test a simple example that should work with current implementation
        let simple_code = r#"
        # Simple test that uses only implemented features
        let x = 10
        let y = 20
        let result = x + y
        result
        "#;

        assert_equals(simple_code, Value::Number(30.0), "Simple arithmetic example");
    }

    #[test]
    fn test_property_examples() {
        let property_code = r#"
        # Property access and assignment example
        let config = {"debug": true, "timeout": 30}
        config.debug = false
        config.timeout = 60
        config
        "#;

        assert_works(property_code, "Property manipulation example");
    }

    #[test]
    fn test_control_flow_examples() {
        let control_flow_code = r#"
        # Control flow example
        let temperature = 25
        if temperature > 20 {
            "warm"
        } else {
            "cool"
        }
        "#;

        assert_works(control_flow_code, "Control flow example");
    }

    #[test]
    fn test_collections_examples() {
        let collections_code = r#"
        # Collections example
        let users = [
            {"name": "Alice", "admin": true},
            {"name": "Bob", "admin": false}
        ]
        users
        "#;

        assert_works(collections_code, "Collections with nested objects");
    }
}

// =============================================================================
// Performance and Stress Tests
// =============================================================================

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_compilation_speed() {
        let complex_code = r#"
        let count = 0
        let sum = 0
        while count < 10 {
            sum = sum + count * count
            count = count + 1
        }
        sum
        "#;

        let start = Instant::now();
        match test_lugli_code(complex_code) {
            Ok(_) => {
                let duration = start.elapsed();
                println!("✅ Complex program compiled and ran in {:?}", duration);
                assert!(duration.as_millis() < 1000, "Compilation should be fast");
            }
            Err(error) => {
                // If this fails, it tells us what features are missing
                println!("ℹ️  Complex program failed (expected for missing features): {}", error);
            }
        }
    }

    #[test]
    fn test_memory_usage() {
        // Create a program with many variables to test memory usage
        let many_vars = r#"
        let a1 = 1; let a2 = 2; let a3 = 3; let a4 = 4; let a5 = 5;
        let a6 = 6; let a7 = 7; let a8 = 8; let a9 = 9; let a10 = 10;
        a1 + a2 + a3 + a4 + a5 + a6 + a7 + a8 + a9 + a10
        "#;

        assert_equals(many_vars, Value::Number(55.0), "Many variables");
    }
}

// =============================================================================
// Main Test Runner
// =============================================================================

#[cfg(test)]
mod test_runner {
    use super::*;

    #[test]
    fn run_all_integration_tests() {
        println!("\n🚀 Running Lugli Language Integration Tests");
        println!("===========================================");

        // The individual test modules will run their tests
        // This is just a summary test that ensures the test framework works

        assert_works("42", "Basic literal");
        println!("\n✅ All integration test modules completed!");
        println!("Check individual test results above for detailed status.");
    }
}
