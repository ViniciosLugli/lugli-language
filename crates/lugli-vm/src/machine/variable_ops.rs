use super::Machine;
use crate::Bytecode;
use lugli_common::{LugliError, Value};

// Variable operation instruction handlers
impl Machine {
    pub(super) fn exec_load(&mut self, index: usize) -> Result<(), LugliError> {
        let frame = self.context.current_frame().ok_or_else(|| LugliError::runtime("Call stack is empty"))?;
        let target_index = frame.stack_base + index;
        let value = self
            .context.stack()
            .get(target_index)
            .ok_or_else(|| {
                LugliError::runtime(format!(
                    "Stack underflow: attempted to load local {} at index {} (stack size: {})",
                    index,
                    target_index,
                    self.context.stack_len()
                ))
            })?
            .clone();
        self.push(value);
        Ok(())
    }

    pub(super) fn exec_store(&mut self, index: usize) -> Result<(), LugliError> {
        let value = self.pop()?;
        let frame = self.context.current_frame().ok_or_else(|| LugliError::runtime("Call stack is empty"))?;
        let target_index = frame.stack_base + index;

        // Grow stack if necessary
        while self.context.stack_len() <= target_index {
            self.push(Value::Null);
        }

        self.context.stack_mut()[target_index] = value;
        Ok(())
    }

    pub(super) fn exec_load_upvalue(&mut self, index: usize) -> Result<(), LugliError> {
        let frame = self.context.current_frame().ok_or_else(|| LugliError::runtime("Call stack is empty"))?;

        if let Some(upvalues) = &frame.closure_upvalues {
            let upvalue_ref = upvalues
                .get(index)
                .ok_or_else(|| LugliError::runtime(format!("Upvalue index {} out of bounds (have {} upvalues)", index, upvalues.len())))?;

            let value = upvalue_ref.try_borrow().map_err(|_| LugliError::runtime("Cannot access upvalue while it's being modified"))?.clone();

            self.push(value);
            Ok(())
        } else {
            Err(LugliError::runtime("LoadUpvalue used in non-closure context"))
        }
    }

    pub(super) fn exec_store_upvalue(&mut self, index: usize) -> Result<(), LugliError> {
        let value = self.pop()?;
        let frame = self.context.current_frame().ok_or_else(|| LugliError::runtime("Call stack is empty"))?;

        if let Some(upvalues) = &frame.closure_upvalues {
            let upvalue_ref = upvalues
                .get(index)
                .ok_or_else(|| LugliError::runtime(format!("Upvalue index {} out of bounds (have {} upvalues)", index, upvalues.len())))?;

            *upvalue_ref.try_borrow_mut().map_err(|_| LugliError::runtime("Cannot modify upvalue while it's being used"))? = value;
            Ok(())
        } else {
            Err(LugliError::runtime("StoreUpvalue used in non-closure context"))
        }
    }

    pub(super) fn exec_load_global(&mut self, bytecode: &Bytecode, name_index: usize) -> Result<(), LugliError> {
        let var_name = self.get_constant(bytecode, name_index)?;
        if let Value::String(name_id) = var_name {
            let name = bytecode.string_pool.borrow().resolve(*name_id).to_string();
            if let Some(value) = self.context.get_global(&name) {
                self.push(value.clone());
                Ok(())
            } else {
                // Generate helpful error with suggestions
                if let Some(context) = self.get_source_context(bytecode) {
                    let available_names: Vec<&str> = self.context.globals().keys().map(|s| s.as_str()).collect();
                    let suggestion = crate::error_formatter::suggest_similar_name(&name, &available_names);
                    Err(LugliError::undefined_variable_with_context(name, context, suggestion))
                } else {
                    Err(LugliError::undefined_variable(name))
                }
            }
        } else {
            Err(LugliError::runtime("Variable name must be a string"))
        }
    }

    pub(super) fn exec_store_global(&mut self, bytecode: &Bytecode, name_index: usize) -> Result<(), LugliError> {
        let value = self.peek()?.clone_for_stack(); // Don't pop - leave value on stack like Store does
        let var_name = self.get_constant(bytecode, name_index)?;
        if let Value::String(name_id) = var_name {
            let name = bytecode.string_pool.borrow().resolve(*name_id).to_string();
            self.context.define_global(name, value);
            Ok(())
        } else {
            Err(LugliError::runtime("Variable name must be a string"))
        }
    }
}
