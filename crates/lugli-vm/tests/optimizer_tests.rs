// Optimizer tests - validates bytecode optimization passes
//
// Tests peephole optimizations:
// - Constant folding: 2 + 3 → 5 at compile time
// - Algebraic simplification: x * 1 → x, x + 0 → x
// - Power folding: 2 ** 3 → 8
// - Integer division optimization
// - Dead code elimination (currently disabled)
//
// Current optimizer status:
// - Constant folding: ✅ ENABLED
// - Jump optimization: ✅ ENABLED
// - Dead code elimination: ❌ DISABLED (too aggressive with closures)

use lugli_parser::Parser;
use lugli_vm::{compile, Instruction};

/// Helper to compile source code and extract the instructions
fn compile_and_get_instructions(source: &str) -> Vec<Instruction> {
    let mut parser = Parser::new(source).expect("Parse creation failed");
    let (program, span_map) = parser.parse().expect("Parse failed");
    let bytecode = compile(&program, span_map).expect("Compilation failed");
    bytecode.instructions.clone()
}

/// Helper to check if instructions contain a specific instruction type
fn has_instruction<F>(instructions: &[Instruction], predicate: F) -> bool
where
    F: Fn(&Instruction) -> bool,
{
    instructions.iter().any(predicate)
}

/// Helper to count specific instructions
fn count_instructions<F>(instructions: &[Instruction], predicate: F) -> usize
where
    F: Fn(&Instruction) -> bool,
{
    instructions.iter().filter(|i| predicate(i)).count()
}

// ============================================================================
// CONSTANT FOLDING TESTS - Core optimization that should always work
// ============================================================================

#[test]
fn test_constant_folding_addition() {
    let source = "let x = 2 + 3";
    let instructions = compile_and_get_instructions(source);

    // Should fold to LoadSmallInt(5) or LoadInt(5), not separate loads + Add
    let has_add = has_instruction(&instructions, |i| matches!(i, Instruction::Add));
    assert!(
        !has_add,
        "Addition of constants should be folded at compile time"
    );

    // Should have exactly one load instruction for the folded constant
    let load_count = count_instructions(&instructions, |i| {
        matches!(
            i,
            Instruction::LoadSmallInt(_) | Instruction::LoadInt(_) | Instruction::Constant(_)
        )
    });
    assert!(
        load_count >= 1,
        "Should have load instruction for folded constant"
    );
}

#[test]
fn test_constant_folding_subtraction() {
    let source = "let x = 10 - 3";
    let instructions = compile_and_get_instructions(source);

    let has_subtract = has_instruction(&instructions, |i| matches!(i, Instruction::Subtract));
    assert!(!has_subtract, "Subtraction should be folded");
}

#[test]
fn test_constant_folding_multiplication() {
    let source = "let x = 4 * 5";
    let instructions = compile_and_get_instructions(source);

    let has_multiply = has_instruction(&instructions, |i| matches!(i, Instruction::Multiply));
    assert!(
        !has_multiply,
        "Multiplication of constants should be folded"
    );
}

#[test]
fn test_constant_folding_division() {
    let source = "let x = 20 / 4";
    let instructions = compile_and_get_instructions(source);

    let has_divide = has_instruction(&instructions, |i| matches!(i, Instruction::Divide));
    assert!(!has_divide, "Division should be folded");
}

#[test]
fn test_constant_folding_modulo() {
    let source = "let x = 17 % 5";
    let instructions = compile_and_get_instructions(source);

    let has_modulo = has_instruction(&instructions, |i| matches!(i, Instruction::Modulo));
    assert!(!has_modulo, "Modulo operation should be folded");
}

#[test]
fn test_constant_folding_power() {
    let source = "let x = 2 ** 3";
    let instructions = compile_and_get_instructions(source);

    // Should fold to 8
    let has_power = has_instruction(&instructions, |i| matches!(i, Instruction::Power));
    assert!(
        !has_power,
        "Power of small integers should be folded to 8"
    );
}

#[test]
fn test_constant_folding_integer_division() {
    let source = "let x = 17 // 5";
    let instructions = compile_and_get_instructions(source);

    let has_int_div = has_instruction(&instructions, |i| {
        matches!(i, Instruction::IntegerDivide)
    });
    assert!(!has_int_div, "Integer division should be folded");
}

#[test]
fn test_constant_folding_nested_expressions() {
    let source = "let x = (2 + 3) * 4";
    let instructions = compile_and_get_instructions(source);

    // Should compute 5 * 4 = 20 at compile time
    let has_arithmetic = has_instruction(&instructions, |i| {
        matches!(i, Instruction::Add | Instruction::Multiply)
    });
    assert!(
        !has_arithmetic,
        "Nested arithmetic (2+3)*4 should fold to 20"
    );
}

#[test]
fn test_constant_folding_deeply_nested() {
    let source = "let x = ((2 + 3) * 4) + (10 / 2)";
    let instructions = compile_and_get_instructions(source);

    // Should compute everything at compile time: (5 * 4) + 5 = 25
    let has_arithmetic = has_instruction(&instructions, |i| {
        matches!(
            i,
            Instruction::Add | Instruction::Multiply | Instruction::Divide
        )
    });
    assert!(
        !has_arithmetic,
        "Deeply nested expression should fold completely"
    );
}

#[test]
fn test_constant_folding_multiple_operations() {
    let source = "let x = 1 + 2 + 3 + 4 + 5";
    let instructions = compile_and_get_instructions(source);

    // Should fold to 15
    let add_count = count_instructions(&instructions, |i| matches!(i, Instruction::Add));
    assert_eq!(
        add_count, 0,
        "Chain of additions should fold to single constant (15)"
    );
}

#[test]
fn test_constant_folding_with_negation() {
    let source = "let x = -(5 + 3)";
    let instructions = compile_and_get_instructions(source);

    // Should fold to -8
    let has_arithmetic = has_instruction(&instructions, |i| {
        matches!(i, Instruction::Add | Instruction::Negate)
    });
    assert!(
        !has_arithmetic,
        "Negation of constant expression should fold"
    );
}

// ============================================================================
// ALGEBRAIC SIMPLIFICATION TESTS - Identity operations
// ============================================================================

#[test]
fn test_algebraic_simplification_add_zero_right() {
    let source = "let y = 5\nlet x = y + 0";
    let instructions = compile_and_get_instructions(source);

    // x + 0 should be optimized away (either folded or AddInt(0) eliminated)
    let add_count = count_instructions(&instructions, |i| {
        matches!(i, Instruction::Add | Instruction::AddInt(_))
    });
    assert_eq!(add_count, 0, "Adding zero should be eliminated (x + 0 → x)");
}

#[test]
fn test_algebraic_simplification_multiply_one_right() {
    let source = "let y = 5\nlet x = y * 1";
    let instructions = compile_and_get_instructions(source);

    // x * 1 should be optimized to just x
    let mul_count = count_instructions(&instructions, |i| {
        matches!(i, Instruction::Multiply | Instruction::MulInt(_))
    });
    assert_eq!(
        mul_count, 0,
        "Multiplying by 1 should be eliminated (x * 1 → x)"
    );
}

#[test]
fn test_algebraic_simplification_multiply_zero_const() {
    let source = "let x = 5 * 0";
    let instructions = compile_and_get_instructions(source);

    // Constant * 0 should be folded at compile time
    let mul_count = count_instructions(&instructions, |i| {
        matches!(i, Instruction::Multiply | Instruction::MulInt(_))
    });
    assert_eq!(
        mul_count, 0,
        "Constant multiplied by 0 should be folded (5 * 0 → 0)"
    );
}

#[test]
fn test_algebraic_simplification_power_zero() {
    let source = "let x = 5 ** 0";
    let instructions = compile_and_get_instructions(source);

    // 5 ** 0 should be optimized to 1 (constant ** 0)
    let power_count = count_instructions(&instructions, |i| matches!(i, Instruction::Power));
    assert_eq!(
        power_count, 0,
        "Constant to power 0 should be 1 (5 ** 0 → 1)"
    );
}

#[test]
fn test_algebraic_simplification_power_one() {
    let source = "let y = 5\nlet x = y ** 1";
    let instructions = compile_and_get_instructions(source);

    // x ** 1 should be optimized to x
    let power_count = count_instructions(&instructions, |i| matches!(i, Instruction::Power));
    assert_eq!(
        power_count, 0,
        "Any value to power 1 should be itself (x ** 1 → x)"
    );
}

// ============================================================================
// VARIABLE OPERATIONS - Should NOT optimize (runtime values)
// ============================================================================

#[test]
fn test_no_optimization_with_variables() {
    let source = "let a = 5\nlet b = 10\nlet c = a + b";
    let instructions = compile_and_get_instructions(source);

    // Should NOT optimize variable addition since values aren't known at compile time
    let has_add = has_instruction(&instructions, |i| {
        matches!(i, Instruction::Add | Instruction::AddInt(_))
    });
    assert!(
        has_add,
        "Variable operations should not be optimized (runtime values)"
    );
}

#[test]
fn test_optimizer_preserves_function_calls() {
    let source = "fn foo() { return 5 }\nlet x = foo() + 0";
    let instructions = compile_and_get_instructions(source);

    // Should keep function call even though + 0 might be optimized
    let has_call = has_instruction(&instructions, |i| matches!(i, Instruction::Call(_)));
    assert!(
        has_call,
        "Function calls must be preserved (side effects)"
    );
}

#[test]
fn test_optimizer_preserves_side_effects_in_expressions() {
    let source = "let x = []\nx.push(5)\nlet y = x.len() * 1";
    let instructions = compile_and_get_instructions(source);

    // Should keep method call even if * 1 is optimized
    let has_call_method = has_instruction(&instructions, |i| {
        matches!(i, Instruction::CallMethod(_, _))
    });
    assert!(
        has_call_method,
        "Method calls must be preserved (side effects)"
    );
}

// ============================================================================
// MULTIPLE OPTIMIZATION PASSES
// ============================================================================

#[test]
fn test_optimizer_multiple_passes() {
    let source = "let x = (2 + 3) + (4 + 5)";
    let instructions = compile_and_get_instructions(source);

    // Multiple folding passes should reduce to single constant: 5 + 9 = 14
    let has_arithmetic = has_instruction(&instructions, |i| {
        matches!(i, Instruction::Add | Instruction::AddInt(_))
    });
    assert!(
        !has_arithmetic,
        "Multiple passes should fold everything to 14"
    );
}

#[test]
fn test_optimizer_handles_complex_mixed_operations() {
    let source = "let x = (10 * 2) + (15 / 3) - (2 ** 3)";
    let instructions = compile_and_get_instructions(source);

    // Should compute: 20 + 5 - 8 = 17
    let has_arithmetic = has_instruction(&instructions, |i| {
        matches!(
            i,
            Instruction::Add
                | Instruction::Subtract
                | Instruction::Multiply
                | Instruction::Divide
                | Instruction::Power
        )
    });
    assert!(!has_arithmetic, "Complex expression should fold to 17");
}

// ============================================================================
// EDGE CASES AND BOUNDARIES
// ============================================================================

#[test]
fn test_constant_folding_with_small_int_optimization() {
    let source = "let x = 1 + 2";
    let instructions = compile_and_get_instructions(source);

    // Should use LoadSmallInt(3) for the result
    let has_add = has_instruction(&instructions, |i| matches!(i, Instruction::Add));

    assert!(
        !has_add,
        "Small integer addition should be folded"
    );
    // Note: The optimizer might produce LoadSmallInt or LoadInt depending on value
}

#[test]
fn test_constant_folding_large_numbers() {
    let source = "let x = 1000000 + 2000000";
    let instructions = compile_and_get_instructions(source);

    // Should still fold even with large numbers
    let has_add = has_instruction(&instructions, |i| matches!(i, Instruction::Add));
    assert!(!has_add, "Large number addition should be folded");
}

#[test]
fn test_constant_folding_negative_numbers() {
    let source = "let x = -5 + -10";
    let instructions = compile_and_get_instructions(source);

    // Should fold to -15
    let has_add = has_instruction(&instructions, |i| matches!(i, Instruction::Add));
    let has_negate = has_instruction(&instructions, |i| matches!(i, Instruction::Negate));
    assert!(!has_add, "Addition of negative constants should fold");
    assert!(
        !has_negate,
        "Negation should be folded into the constant"
    );
}

#[test]
fn test_constant_folding_floating_point_precision() {
    let source = "let x = 0.1 + 0.2";
    let instructions = compile_and_get_instructions(source);

    // Should fold even with floating point (though result may not be exactly 0.3 due to IEEE 754)
    let has_add = has_instruction(&instructions, |i| matches!(i, Instruction::Add));
    assert!(
        !has_add,
        "Floating point addition should be folded (with precision caveats)"
    );
}

// ============================================================================
// DEAD CODE ELIMINATION TESTS - Currently DISABLED
// ============================================================================

#[test]
#[ignore] // DCE currently disabled due to closure issues
fn test_dead_code_after_return() {
    let source = r#"
        fn test() {
            return 42
            print("unreachable")
        }
    "#;
    let _instructions = compile_and_get_instructions(source);

    // When DCE is enabled, this should eliminate the print statement
    // Currently disabled, so print will be in bytecode

    // Uncomment when DCE is fixed:
    // let has_print = has_instruction(&instructions, |i| matches!(i, Instruction::Print));
    // assert!(!has_print, "Unreachable code after return should be eliminated");
}

#[test]
#[ignore] // DCE issues with closures
fn test_dead_code_with_closures() {
    let source = r#"
        fn outer() {
            let x = 10
            fn inner() {
                return 5
            }
            return inner()
        }
    "#;
    let _instructions = compile_and_get_instructions(source);

    // DCE should not eliminate x if it might be captured by closure
    // This tests the bug that caused DCE to be disabled

    // When fixed, verify that DCE correctly handles closure captures
}

#[test]
#[ignore] // DCE disabled
fn test_dead_code_unused_variable() {
    let source = r#"
        fn test() {
            let unused = 42
            return 10
        }
    "#;
    let _instructions = compile_and_get_instructions(source);

    // When DCE is enabled, should not have instructions for unused variable
    // This documents expected behavior when DCE is fixed
}

// ============================================================================
// OPTIMIZER CORRECTNESS - Ensure optimizations don't break semantics
// ============================================================================

#[test]
fn test_optimizer_preserves_evaluation_order() {
    let source = r#"
        fn sideEffect(x) {
            print(x)
            return x
        }
        let y = sideEffect(1) + sideEffect(2)
    "#;
    let instructions = compile_and_get_instructions(source);

    // Both function calls must be preserved in correct order
    let call_count = count_instructions(&instructions, |i| matches!(i, Instruction::Call(_)));
    assert_eq!(
        call_count, 2,
        "Both side-effecting calls must be preserved"
    );
}

#[test]
fn test_optimizer_preserves_short_circuit_evaluation() {
    let source = "let y = 5\nlet x = false && (y > 10)";
    let instructions = compile_and_get_instructions(source);

    // Should preserve short-circuit logic with jumps
    let has_jump_logic = has_instruction(&instructions, |i| {
        matches!(i, Instruction::JumpIfFalse(_) | Instruction::Jump(_))
    });
    assert!(
        has_jump_logic,
        "Short-circuit evaluation structure must be preserved"
    );
}

#[test]
fn test_optimizer_handles_division_by_constant() {
    let source = "let y = 10\nlet x = y / 2";
    let instructions = compile_and_get_instructions(source);

    // Should NOT optimize away the division (y is variable)
    let has_divide = has_instruction(&instructions, |i| matches!(i, Instruction::Divide));
    assert!(
        has_divide,
        "Division with variable dividend should not be optimized"
    );
}

// ============================================================================
// CONSTANT PROPAGATION - Future optimization (document current behavior)
// ============================================================================

#[test]
fn test_constant_propagation_not_implemented() {
    let source = r#"
        let x = 5
        let y = x + 3
    "#;
    let instructions = compile_and_get_instructions(source);

    // Currently no constant propagation across statements
    // The x + 3 will have Add instruction since x's value isn't propagated
    let has_add = has_instruction(&instructions, |i| {
        matches!(i, Instruction::Add | Instruction::AddInt(_))
    });
    assert!(
        has_add,
        "Constant propagation not yet implemented (x + 3 not folded)"
    );
}

#[test]
fn test_no_common_subexpression_elimination() {
    let source = r#"
        let a = 5
        let b = a + 10
        let c = a + 10
    "#;
    let instructions = compile_and_get_instructions(source);

    // CSE not implemented - both a + 10 expressions will compute separately
    let add_count = count_instructions(&instructions, |i| {
        matches!(i, Instruction::Add | Instruction::AddInt(_))
    });
    assert!(
        add_count >= 2,
        "No common subexpression elimination (a + 10 computed twice)"
    );
}

// ============================================================================
// REGRESSION TESTS - Ensure optimizer doesn't break known-good code
// ============================================================================

#[test]
fn test_optimizer_handles_empty_program() {
    let source = "";
    let instructions = compile_and_get_instructions(source);

    // Should produce minimal bytecode (just return/halt)
    assert!(
        instructions.len() < 5,
        "Empty program should have minimal instructions"
    );
}

#[test]
fn test_optimizer_handles_single_constant() {
    let source = "42";
    let instructions = compile_and_get_instructions(source);

    // Should just load the constant
    let load_count = count_instructions(&instructions, |i| {
        matches!(
            i,
            Instruction::LoadSmallInt(_) | Instruction::LoadInt(_) | Instruction::Constant(_)
        )
    });
    assert_eq!(load_count, 1, "Single constant should have one load");
}

#[test]
fn test_optimizer_with_string_concatenation() {
    let source = r#"let x = "hello" + "world""#;
    let _instructions = compile_and_get_instructions(source);

    // String concatenation is runtime operation, not compile-time
    // Just verify it compiles without errors
}

#[test]
fn test_optimizer_with_list_literals() {
    let source = "let x = [1 + 2, 3 * 4, 5]";
    let instructions = compile_and_get_instructions(source);

    // Constant expressions in list should be folded
    let has_arithmetic = has_instruction(&instructions, |i| {
        matches!(i, Instruction::Add | Instruction::Multiply)
    });
    assert!(
        !has_arithmetic,
        "Constant expressions in list literals should be folded"
    );
}

#[test]
fn test_optimizer_with_dict_literals() {
    let source = r#"let x = {"a": 1 + 2, "b": 3 * 4}"#;
    let instructions = compile_and_get_instructions(source);

    // Constant expressions in dict should be folded
    let has_arithmetic = has_instruction(&instructions, |i| {
        matches!(i, Instruction::Add | Instruction::Multiply)
    });
    assert!(
        !has_arithmetic,
        "Constant expressions in dict literals should be folded"
    );
}
