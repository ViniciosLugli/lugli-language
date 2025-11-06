use crate::{Bytecode, Instruction};

pub fn disassemble(bytecode: &Bytecode, name: &str) -> String {
    let mut output = format!("== {} ==\n", name);
    output.push_str("Offset | Line | Opcode          | Operands\n");
    output.push_str("----------------------------------------\n");

    let mut i = 0;
    while i < bytecode.instructions.len() {
        output.push_str(&disassemble_instruction(bytecode, i));
        i += 1;
    }

    output
}

fn disassemble_instruction(bytecode: &Bytecode, offset: usize) -> String {
    let instruction = match bytecode.instructions.get(offset) {
        Some(instr) => instr,
        None => return format!("{:04}   | {:<4} | INVALID_OFFSET  | offset {} out of bounds\n", offset, "?", offset),
    };
    let op_str = format!("{:?}", instruction);

    let (operands, _size) = match instruction {
        Instruction::Constant(idx) => {
            let const_display =
                bytecode.constants.get(*idx).map(|c| format!("{} ('{:?}')", idx, c)).unwrap_or_else(|| format!("{} (invalid constant index)", idx));
            (const_display, 2)
        }
        Instruction::Load(idx) => (format!("{}", idx), 2),
        Instruction::Store(idx) => (format!("{}", idx), 2),
        Instruction::ReserveLocals(count) => (format!("{}", count), 2),
        Instruction::LoadGlobal(idx) => {
            let const_display = bytecode.constants.get(*idx).map(|c| format!("{} ('{:?}')", idx, c)).unwrap_or_else(|| format!("{} (invalid)", idx));
            (const_display, 2)
        }
        Instruction::StoreGlobal(idx) => {
            let const_display = bytecode.constants.get(*idx).map(|c| format!("{} ('{:?}')", idx, c)).unwrap_or_else(|| format!("{} (invalid)", idx));
            (const_display, 2)
        }
        Instruction::GetProperty(idx) => {
            let const_display = bytecode.constants.get(*idx).map(|c| format!("{} ('{:?}')", idx, c)).unwrap_or_else(|| format!("{} (invalid)", idx));
            (const_display, 2)
        }
        Instruction::SetProperty(idx) => {
            let const_display = bytecode.constants.get(*idx).map(|c| format!("{} ('{:?}')", idx, c)).unwrap_or_else(|| format!("{} (invalid)", idx));
            (const_display, 2)
        }
        Instruction::Call(arg_count) => (format!("{}", arg_count), 2),
        Instruction::Jump(addr) => (format!("{:04}", addr), 2),
        Instruction::JumpIfFalse(addr) => (format!("{:04}", addr), 2),
        Instruction::Loop(addr) => (format!("{:04}", addr), 2),
        Instruction::DefineFunction(idx) => {
            let const_display = bytecode.constants.get(*idx).map(|c| format!("{} ('{:?}')", idx, c)).unwrap_or_else(|| format!("{} (invalid)", idx));
            (const_display, 2)
        }
        Instruction::ImportModule {
            module_idx,
            bind_name,
        } => {
            let const_display = bytecode
                .constants
                .get(*module_idx)
                .map(|c| format!("{} as '{}'", c, bind_name))
                .unwrap_or_else(|| format!("invalid_idx:{} as '{}'", module_idx, bind_name));
            (const_display, 1)
        }
        Instruction::ImportFrom {
            module_idx,
            names,
        } => {
            let const_display = bytecode
                .constants
                .get(*module_idx)
                .map(|c| format!("{} import {:?}", c, names))
                .unwrap_or_else(|| format!("invalid_idx:{} import {:?}", module_idx, names));
            (const_display, 1)
        }
        _ => ("".to_string(), 1),
    };

    format!("{:04}   | {:<4} | {:<15} | {}\n", offset, "?", op_str, operands)
}
