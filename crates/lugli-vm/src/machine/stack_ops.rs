use crate::{Bytecode, Instruction};
use lugli_common::{LugliError, Value};
use super::Machine;

// Stack operation instruction handlers
impl Machine {
    pub(super) fn exec_constant(&mut self, bytecode: &Bytecode, index: usize) -> Result<(), LugliError> {
        let value = self.get_constant(bytecode, index)?.clone_for_stack();
        self.stack.push(value);
        Ok(())
    }

    pub(super) fn exec_load_small_int(&mut self, n: i8) -> Result<(), LugliError> {
        self.stack.push(Value::Number(n as f64));
        Ok(())
    }

    pub(super) fn exec_load_int(&mut self, n: i16) -> Result<(), LugliError> {
        self.stack.push(Value::Number(n as f64));
        Ok(())
    }

    pub(super) fn exec_load_true(&mut self) -> Result<(), LugliError> {
        self.stack.push(Value::Bool(true));
        Ok(())
    }

    pub(super) fn exec_load_false(&mut self) -> Result<(), LugliError> {
        self.stack.push(Value::Bool(false));
        Ok(())
    }

    pub(super) fn exec_load_null(&mut self) -> Result<(), LugliError> {
        self.stack.push(Value::Null);
        Ok(())
    }

    pub(super) fn exec_reserve_locals(&mut self, count: usize) -> Result<(), LugliError> {
        for _ in 0..count {
            self.stack.push(Value::Null);
        }
        Ok(())
    }

    pub(super) fn exec_pop(&mut self) -> Result<(), LugliError> {
        self.pop()?;
        Ok(())
    }

    pub(super) fn exec_dup(&mut self) -> Result<(), LugliError> {
        let val = self.peek()?.clone_for_stack();
        self.stack.push(val);
        Ok(())
    }

    pub(super) fn exec_print(&mut self) -> Result<(), LugliError> {
        let val = self.pop()?;
        println!("{}", val);
        Ok(())
    }
}
