use super::Machine;
use crate::Bytecode;
use lugli_common::{LugliError, Value};

// Arithmetic and logical operation instruction handlers
impl Machine {
    pub(super) fn exec_add(&mut self, bytecode: &Bytecode) -> Result<(), LugliError> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (&a, &b) {
            (Value::String(_), Value::String(_)) => {
                let mut pool = bytecode.string_pool.borrow_mut();
                a.add_with_pool(&b, &mut pool)?
            }
            _ => a.add(&b)?,
        };
        self.push(result);
        Ok(())
    }

    pub(super) fn exec_subtract(&mut self) -> Result<(), LugliError> { self.execute_binary_op(Value::subtract) }

    pub(super) fn exec_multiply(&mut self) -> Result<(), LugliError> { self.execute_binary_op(Value::multiply) }

    pub(super) fn exec_divide(&mut self) -> Result<(), LugliError> { self.execute_binary_op(Value::divide) }

    pub(super) fn exec_integer_divide(&mut self) -> Result<(), LugliError> { self.execute_binary_op(Value::integer_divide) }

    pub(super) fn exec_modulo(&mut self) -> Result<(), LugliError> { self.execute_binary_op(Value::modulo) }

    pub(super) fn exec_power(&mut self) -> Result<(), LugliError> { self.execute_binary_op(Value::power) }

    pub(super) fn exec_add_int(&mut self, n: i8) -> Result<(), LugliError> {
        let left = self.pop()?;
        if let Value::Number(a) = left {
            self.push(Value::Number(a + (n as f64)));
            Ok(())
        } else {
            Err(LugliError::type_error("number", left.type_name()))
        }
    }

    pub(super) fn exec_sub_int(&mut self, n: i8) -> Result<(), LugliError> {
        let left = self.pop()?;
        if let Value::Number(a) = left {
            self.push(Value::Number(a - (n as f64)));
            Ok(())
        } else {
            Err(LugliError::type_error("number", left.type_name()))
        }
    }

    pub(super) fn exec_mul_int(&mut self, n: i8) -> Result<(), LugliError> {
        let left = self.pop()?;
        if let Value::Number(a) = left {
            self.push(Value::Number(a * (n as f64)));
            Ok(())
        } else {
            Err(LugliError::type_error("number", left.type_name()))
        }
    }

    pub(super) fn exec_add_locals(&mut self, a_idx: usize, b_idx: usize, bytecode: &Bytecode) -> Result<(), LugliError> {
        let frame = self.context.current_frame().ok_or_else(|| LugliError::runtime("No call frame"))?;
        let base = frame.stack_base;
        let a = self.context.stack().get(base + a_idx).ok_or_else(|| LugliError::runtime(format!("Local {} out of bounds", a_idx)))?.clone();
        let b = self.context.stack().get(base + b_idx).ok_or_else(|| LugliError::runtime(format!("Local {} out of bounds", b_idx)))?.clone();

        let result = match (&a, &b) {
            (Value::String(_), Value::String(_)) => {
                let mut pool = bytecode.string_pool.borrow_mut();
                a.add_with_pool(&b, &mut pool)?
            }
            _ => a.add(&b)?,
        };
        self.push(result);
        Ok(())
    }

    pub(super) fn exec_negate(&mut self) -> Result<(), LugliError> {
        let val = self.pop()?;
        self.push(val.negate()?);
        Ok(())
    }

    pub(super) fn exec_not(&mut self) -> Result<(), LugliError> {
        let val = self.pop()?;
        self.push(Value::Bool(!val.is_truthy()));
        Ok(())
    }

    pub(super) fn exec_equal(&mut self) -> Result<(), LugliError> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(Value::Bool(a.equals(&b)));
        Ok(())
    }

    pub(super) fn exec_not_equal(&mut self) -> Result<(), LugliError> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(Value::Bool(!a.equals(&b)));
        Ok(())
    }

    pub(super) fn exec_greater(&mut self) -> Result<(), LugliError> { self.execute_binary_op(Value::greater) }

    pub(super) fn exec_greater_equal(&mut self) -> Result<(), LugliError> { self.execute_binary_op(Value::greater_equal) }

    pub(super) fn exec_less(&mut self) -> Result<(), LugliError> { self.execute_binary_op(Value::less) }

    pub(super) fn exec_less_equal(&mut self) -> Result<(), LugliError> { self.execute_binary_op(Value::less_equal) }
}
