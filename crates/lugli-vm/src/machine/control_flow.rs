use lugli_common::{LugliError, Value};
use super::Machine;

// Control flow instruction handlers
impl Machine {
    pub(super) fn exec_jump(&mut self, addr: usize) -> Result<bool, LugliError> {
        self.ip = addr;
        Ok(true)
    }

    pub(super) fn exec_jump_if_false(&mut self, addr: usize) -> Result<bool, LugliError> {
        let condition = self.pop()?;
        if !condition.is_truthy() {
            self.ip = addr;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub(super) fn exec_loop(&mut self, start: usize) -> Result<bool, LugliError> {
        self.ip = start;
        Ok(true)
    }

    pub(super) fn exec_return(&mut self) -> Result<bool, LugliError> {
        let frame = self.call_stack.pop().ok_or_else(|| LugliError::runtime("Call stack underflow"))?;
        let return_value = self.pop().unwrap_or(Value::Null);

        // Close upvalues for this frame before returning
        let frame_base = frame.stack_base;
        let frame_end = self.stack.len();
        self.open_upvalues.retain(|&index, _| index < frame_base || index >= frame_end);

        if self.call_stack.is_empty() {
            self.stack.push(return_value);
            return Ok(false);
        }

        self.ip = frame.return_ip;
        self.stack.truncate(frame.stack_base);
        self.stack.push(return_value);
        Ok(true)
    }
}
