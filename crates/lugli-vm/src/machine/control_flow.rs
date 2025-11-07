use super::Machine;
use lugli_common::{LugliError, Value};

// Control flow instruction handlers
impl Machine {
    pub(super) fn exec_jump(&mut self, addr: usize) -> Result<bool, LugliError> {
        self.context.jump_to(addr);
        Ok(true)
    }

    pub(super) fn exec_jump_if_false(&mut self, addr: usize) -> Result<bool, LugliError> {
        let condition = self.pop()?;
        if !condition.is_truthy() {
            self.context.jump_to(addr);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub(super) fn exec_jump_if_equal(&mut self, addr: usize) -> Result<bool, LugliError> {
        let b = self.pop()?;
        let a = self.pop()?;
        if a.equals(&b) {
            self.context.jump_to(addr);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub(super) fn exec_jump_if_not_equal(&mut self, addr: usize) -> Result<bool, LugliError> {
        let b = self.pop()?;
        let a = self.pop()?;
        if !a.equals(&b) {
            self.context.jump_to(addr);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub(super) fn exec_loop(&mut self, start: usize) -> Result<bool, LugliError> {
        self.context.jump_to(start);
        Ok(true)
    }

    pub(super) fn exec_return(&mut self) -> Result<bool, LugliError> {
        let frame = self.context.pop_frame().ok_or_else(|| LugliError::runtime("Call stack underflow"))?;
        let return_value = self.pop().unwrap_or(Value::Null);

        // Close upvalues for this frame before returning
        let frame_base = frame.stack_base;
        let frame_end = self.context.stack_len();
        self.context.close_upvalues(frame_base.min(frame_end));

        if self.context.call_depth() == 0 {
            self.push(return_value);
            return Ok(false);
        }

        self.context.jump_to(frame.return_ip);
        self.context.truncate_stack(frame.stack_base);
        self.push(return_value);
        Ok(true)
    }
}
