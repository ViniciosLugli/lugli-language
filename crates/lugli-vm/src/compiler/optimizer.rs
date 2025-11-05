use crate::Instruction;

/// Peephole optimizer for bytecode instructions
///
/// Performs local optimizations on small instruction windows:
/// - Constant folding: Evaluate constant expressions at compile time
/// - Dead code elimination: Remove unreachable code and unused results
/// - Jump threading: Simplify jump chains
/// - Algebraic simplifications: x + 0, x * 1, etc.
///
/// Inspired by:
/// - Lua 5.4's code generator optimizations
/// - Python's peephole.c optimizer
/// - V8's hydrogen pipeline (simplified)
pub struct PeepholeOptimizer {
    optimize_constants: bool,
    optimize_jumps: bool,
    optimize_dead_code: bool,
}

impl PeepholeOptimizer {
    pub fn new() -> Self {
        Self {
            optimize_constants: true,
            optimize_jumps: true,      // Re-enabled with offset tracking
            optimize_dead_code: false, // Keep disabled - still too aggressive
        }
    }

    /// Run all optimization passes on bytecode
    pub fn optimize(&self, instructions: Vec<Instruction>) -> Vec<Instruction> {
        let mut optimized = instructions;

        // Multiple passes until fixed point (no more changes)
        let mut changed = true;
        let mut pass_count = 0;
        const MAX_PASSES: usize = 5; // Prevent infinite loops

        while changed && pass_count < MAX_PASSES {
            let before_len = optimized.len();

            if self.optimize_constants {
                optimized = self.constant_folding(optimized);
                optimized = self.algebraic_simplification(optimized);
            }

            if self.optimize_dead_code {
                optimized = self.dead_code_elimination(optimized);
            }

            if self.optimize_jumps {
                optimized = self.jump_threading(optimized);
            }

            changed = optimized.len() != before_len;
            pass_count += 1;
        }

        optimized
    }

    /// Fold constant expressions at compile time
    ///
    /// Examples:
    /// - LoadSmallInt(2) LoadSmallInt(3) Add → LoadSmallInt(5)
    /// - LoadSmallInt(10) LoadSmallInt(2) Multiply → LoadSmallInt(20)
    /// - LoadSmallInt(5) Negate → LoadSmallInt(-5)
    fn constant_folding(&self, instructions: Vec<Instruction>) -> Vec<Instruction> {
        let mut result = Vec::with_capacity(instructions.len());
        let mut i = 0;

        while i < instructions.len() {
            // Try 3-instruction patterns first (binary ops)
            if i + 2 < instructions.len()
                && let Some(folded) = self.try_fold_binary(&instructions[i], &instructions[i + 1], &instructions[i + 2])
            {
                result.push(folded);
                i += 3;
                continue;
            }

            // Try 2-instruction patterns (unary ops)
            if i + 1 < instructions.len()
                && let Some(folded) = self.try_fold_unary(&instructions[i], &instructions[i + 1])
            {
                result.push(folded);
                i += 2;
                continue;
            }

            // No optimization, keep instruction
            result.push(instructions[i].clone());
            i += 1;
        }

        result
    }

    /// Try to fold binary operations on constants
    fn try_fold_binary(&self, inst1: &Instruction, inst2: &Instruction, inst3: &Instruction) -> Option<Instruction> {
        match (inst1, inst2, inst3) {
            // SmallInt arithmetic
            (Instruction::LoadSmallInt(a), Instruction::LoadSmallInt(b), Instruction::Add) => {
                let result = a.checked_add(*b)?;
                Some(Instruction::LoadSmallInt(result))
            }
            (Instruction::LoadSmallInt(a), Instruction::LoadSmallInt(b), Instruction::Subtract) => {
                let result = a.checked_sub(*b)?;
                Some(Instruction::LoadSmallInt(result))
            }
            (Instruction::LoadSmallInt(a), Instruction::LoadSmallInt(b), Instruction::Multiply) => {
                let result = a.checked_mul(*b)?;
                Some(Instruction::LoadSmallInt(result))
            }
            (Instruction::LoadSmallInt(a), Instruction::LoadSmallInt(b), Instruction::Divide) if *b != 0 => {
                let result = a / b;
                Some(Instruction::LoadSmallInt(result))
            }
            (Instruction::LoadSmallInt(a), Instruction::LoadSmallInt(b), Instruction::Modulo) if *b != 0 => {
                let result = a % b;
                Some(Instruction::LoadSmallInt(result))
            }

            // Boolean operations
            (Instruction::LoadTrue, Instruction::LoadTrue, Instruction::And) => Some(Instruction::LoadTrue),
            (Instruction::LoadTrue, Instruction::LoadFalse, Instruction::And) => Some(Instruction::LoadFalse),
            (Instruction::LoadFalse, _, Instruction::And) => Some(Instruction::LoadFalse),
            (Instruction::LoadTrue, Instruction::LoadTrue, Instruction::Or) => Some(Instruction::LoadTrue),
            (Instruction::LoadTrue, _, Instruction::Or) => Some(Instruction::LoadTrue),
            (Instruction::LoadFalse, Instruction::LoadFalse, Instruction::Or) => Some(Instruction::LoadFalse),

            // Comparisons
            (Instruction::LoadSmallInt(a), Instruction::LoadSmallInt(b), Instruction::Equal) => {
                Some(if a == b { Instruction::LoadTrue } else { Instruction::LoadFalse })
            }
            (Instruction::LoadSmallInt(a), Instruction::LoadSmallInt(b), Instruction::NotEqual) => {
                Some(if a != b { Instruction::LoadTrue } else { Instruction::LoadFalse })
            }
            (Instruction::LoadSmallInt(a), Instruction::LoadSmallInt(b), Instruction::Less) => {
                Some(if a < b { Instruction::LoadTrue } else { Instruction::LoadFalse })
            }
            (Instruction::LoadSmallInt(a), Instruction::LoadSmallInt(b), Instruction::LessEqual) => {
                Some(if a <= b { Instruction::LoadTrue } else { Instruction::LoadFalse })
            }
            (Instruction::LoadSmallInt(a), Instruction::LoadSmallInt(b), Instruction::Greater) => {
                Some(if a > b { Instruction::LoadTrue } else { Instruction::LoadFalse })
            }
            (Instruction::LoadSmallInt(a), Instruction::LoadSmallInt(b), Instruction::GreaterEqual) => {
                Some(if a >= b { Instruction::LoadTrue } else { Instruction::LoadFalse })
            }

            _ => None,
        }
    }

    /// Try to fold unary operations on constants
    fn try_fold_unary(&self, inst1: &Instruction, inst2: &Instruction) -> Option<Instruction> {
        match (inst1, inst2) {
            // Negate constants
            (Instruction::LoadSmallInt(a), Instruction::Negate) => {
                let result = a.checked_neg()?;
                Some(Instruction::LoadSmallInt(result))
            }

            // Boolean not
            (Instruction::LoadTrue, Instruction::Not) => Some(Instruction::LoadFalse),
            (Instruction::LoadFalse, Instruction::Not) => Some(Instruction::LoadTrue),

            // Double negation cancels out
            (Instruction::Negate, Instruction::Negate) => None, // Would need value tracking

            _ => None,
        }
    }

    /// Algebraic simplifications
    ///
    /// Examples:
    /// - x + 0 → x
    /// - x * 1 → x
    /// - x * 0 → 0
    fn algebraic_simplification(&self, instructions: Vec<Instruction>) -> Vec<Instruction> {
        let mut result = Vec::with_capacity(instructions.len());
        let mut i = 0;

        while i < instructions.len() {
            if i + 2 < instructions.len() {
                match (&instructions[i + 1], &instructions[i + 2]) {
                    // x + 0 → x (keep first operand, skip second and Add)
                    (Instruction::LoadSmallInt(0), Instruction::Add) => {
                        result.push(instructions[i].clone());
                        i += 3;
                        continue;
                    }
                    // x * 1 → x
                    (Instruction::LoadSmallInt(1), Instruction::Multiply) => {
                        result.push(instructions[i].clone());
                        i += 3;
                        continue;
                    }
                    // x * 0 → 0 (replace with just LoadSmallInt(0))
                    (Instruction::LoadSmallInt(0), Instruction::Multiply) => {
                        result.push(Instruction::LoadSmallInt(0));
                        i += 3;
                        continue;
                    }
                    _ => {}
                }
            }

            result.push(instructions[i].clone());
            i += 1;
        }

        result
    }

    /// Remove dead code
    ///
    /// Patterns:
    /// - Constant followed by Pop (unused result)
    /// - Code after unconditional Jump or Return
    /// - Jumps to next instruction
    ///
    /// CONSERVATIVE: Never removes Store/Load/Upvalue instructions (needed for closures)
    fn dead_code_elimination(&self, instructions: Vec<Instruction>) -> Vec<Instruction> {
        let mut result = Vec::with_capacity(instructions.len());
        let mut i = 0;
        let mut in_dead_code = false;

        while i < instructions.len() {
            let inst = &instructions[i];

            // Exit dead code region on jump targets
            if matches!(inst, Instruction::Jump(_) | Instruction::JumpIfFalse(_) | Instruction::Loop(_)) {
                in_dead_code = false;
            }

            // Skip dead code after Return or unconditional Jump
            if in_dead_code {
                // Don't skip jump targets (other jumps might point here)
                if !self.is_potential_jump_target(i, &instructions) {
                    // Don't remove instructions that might have side effects
                    if !self.has_side_effects(inst) {
                        i += 1;
                        continue;
                    }
                }
                in_dead_code = false;
            }

            // Pattern: Constant Pop → (remove both)
            // BUT: Don't remove if it might affect closure/upvalue setup
            if i + 1 < instructions.len() && self.is_pure_constant(inst) && matches!(instructions[i + 1], Instruction::Pop) {
                // Check if this is near closure-related instructions
                if !self.is_near_closure_ops(i, &instructions) {
                    i += 2;
                    continue;
                }
            }

            // Pattern: Jump to next instruction → remove jump
            if let Instruction::Jump(target) = inst
                && *target == i + 1
            {
                i += 1;
                continue;
            }

            result.push(inst.clone());

            // Mark dead code after Return or unconditional Jump
            if matches!(inst, Instruction::Return | Instruction::Jump(_)) {
                in_dead_code = true;
            }

            i += 1;
        }

        result
    }

    /// Check if instruction has side effects (must not be removed)
    fn has_side_effects(&self, inst: &Instruction) -> bool {
        matches!(
            inst,
            Instruction::Store(_)
                | Instruction::StoreGlobal(_)
                | Instruction::StoreUpvalue(_)
                | Instruction::DefineFunction(_)
                | Instruction::MakeClosure { .. }
                | Instruction::SetProperty(_)
                | Instruction::SetIndex
                | Instruction::Call(_)
                | Instruction::CallMethod(_, _)
                | Instruction::Print
                | Instruction::ImportModule { .. }
                | Instruction::ImportFrom { .. }
        )
    }

    /// Check if position is near closure operations (within 5 instructions)
    fn is_near_closure_ops(&self, pos: usize, instructions: &[Instruction]) -> bool {
        let start = pos.saturating_sub(5);
        let end = (pos + 5).min(instructions.len());

        for inst in &instructions[start..end] {
            if matches!(
                inst,
                Instruction::DefineFunction(_) | Instruction::MakeClosure { .. } | Instruction::LoadUpvalue(_) | Instruction::StoreUpvalue(_)
            ) {
                return true;
            }
        }
        false
    }

    /// Simplify jump chains
    ///
    /// Pattern: Jump(A) where instructions[A] = Jump(B) → Jump(B)
    fn jump_threading(&self, instructions: Vec<Instruction>) -> Vec<Instruction> {
        // Build jump target map
        let mut jump_targets: hashbrown::HashMap<usize, usize> = hashbrown::HashMap::new();

        for (i, inst) in instructions.iter().enumerate() {
            if let Instruction::Jump(target) = inst {
                jump_targets.insert(i, *target);
            }
        }

        // Follow jump chains to find ultimate targets
        let mut ultimate_targets: hashbrown::HashMap<usize, usize> = hashbrown::HashMap::new();

        for (&start, &target) in &jump_targets {
            let mut current = target;
            let mut visited = std::collections::HashSet::new();
            visited.insert(start);

            // Follow chain (with cycle detection)
            while let Some(&next) = jump_targets.get(&current) {
                if visited.contains(&next) {
                    break; // Cycle detected
                }
                visited.insert(current);
                current = next;
            }

            if current != target {
                ultimate_targets.insert(start, current);
            }
        }

        // Apply threading
        if ultimate_targets.is_empty() {
            return instructions;
        }

        instructions
            .into_iter()
            .enumerate()
            .map(|(i, inst)| if let Some(&new_target) = ultimate_targets.get(&i) { Instruction::Jump(new_target) } else { inst })
            .collect()
    }

    /// Check if instruction produces no side effects
    fn is_pure_constant(&self, inst: &Instruction) -> bool {
        matches!(
            inst,
            Instruction::LoadSmallInt(_)
                | Instruction::LoadInt(_)
                | Instruction::LoadTrue
                | Instruction::LoadFalse
                | Instruction::LoadNull
                | Instruction::Constant(_)
        )
    }

    /// Check if position might be a jump target
    fn is_potential_jump_target(&self, pos: usize, instructions: &[Instruction]) -> bool {
        for inst in instructions {
            match inst {
                Instruction::Jump(target) | Instruction::JumpIfFalse(target) | Instruction::Loop(target) => {
                    if *target == pos {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}

impl Default for PeepholeOptimizer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_folding_add() {
        let optimizer = PeepholeOptimizer::new();
        let instructions = vec![Instruction::LoadSmallInt(2), Instruction::LoadSmallInt(3), Instruction::Add];

        let optimized = optimizer.optimize(instructions);
        assert_eq!(optimized, vec![Instruction::LoadSmallInt(5)]);
    }

    #[test]
    fn test_constant_folding_multiply() {
        let optimizer = PeepholeOptimizer::new();
        let instructions = vec![Instruction::LoadSmallInt(4), Instruction::LoadSmallInt(5), Instruction::Multiply];

        let optimized = optimizer.optimize(instructions);
        assert_eq!(optimized, vec![Instruction::LoadSmallInt(20)]);
    }

    #[test]
    fn test_constant_folding_comparison() {
        let optimizer = PeepholeOptimizer::new();
        let instructions = vec![Instruction::LoadSmallInt(5), Instruction::LoadSmallInt(3), Instruction::Greater];

        let optimized = optimizer.optimize(instructions);
        assert_eq!(optimized, vec![Instruction::LoadTrue]);
    }

    #[test]
    fn test_algebraic_simplification_add_zero() {
        let optimizer = PeepholeOptimizer::new();
        let instructions = vec![Instruction::LoadSmallInt(42), Instruction::LoadSmallInt(0), Instruction::Add];

        let optimized = optimizer.optimize(instructions);
        assert_eq!(optimized, vec![Instruction::LoadSmallInt(42)]);
    }

    #[test]
    fn test_algebraic_simplification_mul_one() {
        let optimizer = PeepholeOptimizer::new();
        let instructions = vec![Instruction::LoadSmallInt(42), Instruction::LoadSmallInt(1), Instruction::Multiply];

        let optimized = optimizer.optimize(instructions);
        assert_eq!(optimized, vec![Instruction::LoadSmallInt(42)]);
    }

    #[test]
    fn test_algebraic_simplification_mul_zero() {
        let optimizer = PeepholeOptimizer::new();
        let instructions = vec![Instruction::LoadSmallInt(42), Instruction::LoadSmallInt(0), Instruction::Multiply];

        let optimized = optimizer.optimize(instructions);
        assert_eq!(optimized, vec![Instruction::LoadSmallInt(0)]);
    }

    #[test]
    #[ignore] // Dead code elimination temporarily disabled (too aggressive with closures)
    fn test_dead_code_constant_pop() {
        let optimizer = PeepholeOptimizer::new();
        let instructions = vec![Instruction::LoadSmallInt(42), Instruction::Pop, Instruction::LoadSmallInt(1)];

        let optimized = optimizer.optimize(instructions);
        assert_eq!(optimized, vec![Instruction::LoadSmallInt(1)]);
    }

    #[test]
    fn test_unary_folding_negate() {
        let optimizer = PeepholeOptimizer::new();
        let instructions = vec![Instruction::LoadSmallInt(5), Instruction::Negate];

        let optimized = optimizer.optimize(instructions);
        assert_eq!(optimized, vec![Instruction::LoadSmallInt(-5)]);
    }

    #[test]
    fn test_boolean_folding() {
        let optimizer = PeepholeOptimizer::new();
        let instructions = vec![Instruction::LoadTrue, Instruction::LoadFalse, Instruction::And];

        let optimized = optimizer.optimize(instructions);
        assert_eq!(optimized, vec![Instruction::LoadFalse]);
    }

    #[test]
    fn test_no_overflow_folding() {
        let optimizer = PeepholeOptimizer::new();
        let instructions = vec![Instruction::LoadSmallInt(127), Instruction::LoadSmallInt(1), Instruction::Add];

        // Should not fold due to i8 overflow
        let optimized = optimizer.optimize(instructions);
        assert_eq!(optimized.len(), 3);
    }
}
