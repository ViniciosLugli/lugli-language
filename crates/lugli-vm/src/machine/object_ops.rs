use crate::Bytecode;
use lugli_common::{LugliError, Value, StringId};
use hashbrown::HashMap;
use std::{cell::RefCell, rc::Rc};
use super::Machine;

// Object property and collection instruction handlers
impl Machine {
    pub(super) fn exec_get_property(&mut self, bytecode: &Bytecode, name_index: usize) -> Result<(), LugliError> {
        let object = self.pop()?;
        let prop_name = self.get_constant(bytecode, name_index)?;
        if let Value::String(name_id) = prop_name {
            match object {
                Value::Dict(dict_ref) => {
                    let dict = dict_ref.borrow();
                    let value = dict.get(name_id).cloned().unwrap_or(Value::Null);
                    self.stack.push(value);
                }
                Value::Module { exports, .. } => {
                    let name = bytecode.string_pool.borrow().resolve(*name_id).to_string();
                    if let Some(value) = exports.get(&name) {
                        self.stack.push(value.clone());
                    } else {
                        return Err(LugliError::runtime(format!("Module has no export '{}'", name)));
                    }
                }
                _ => {
                    let name = bytecode.string_pool.borrow().resolve(*name_id).to_string();
                    return Err(LugliError::runtime(format!(
                        "Cannot access property '{}' on a value of type {}",
                        name,
                        object.type_name()
                    )));
                }
            }
            Ok(())
        } else {
            Err(LugliError::runtime("Property name must be a string"))
        }
    }

    pub(super) fn exec_set_property(&mut self, bytecode: &Bytecode, name_index: usize) -> Result<(), LugliError> {
        let value = self.pop()?;
        let object = self.pop()?;
        let prop_name = self.get_constant(bytecode, name_index)?;

        if let Value::String(name_id) = prop_name {
            if let Value::Dict(dict_ref) = &object {
                // Detect potential circular reference
                if let Value::Dict(value_dict_ref) = &value {
                    if Rc::ptr_eq(dict_ref, value_dict_ref) {
                        eprintln!("⚠️  WARNING: Assigning dictionary to itself creates a circular reference");
                    }
                }

                dict_ref
                    .try_borrow_mut()
                    .map_err(|_| LugliError::runtime("Cannot modify struct while it's being used"))?
                    .insert(*name_id, value.clone());
                self.stack.push(value);
                Ok(())
            } else {
                let name = bytecode.string_pool.borrow().resolve(*name_id).to_string();
                Err(LugliError::runtime(format!(
                    "Cannot set property '{}' on a value of type {}",
                    name,
                    object.type_name()
                )))
            }
        } else {
            Err(LugliError::runtime("Property name must be a string"))
        }
    }

    pub(super) fn exec_make_list(&mut self, count: usize) -> Result<(), LugliError> {
        let mut list = Vec::with_capacity(count);
        for _ in 0..count {
            list.push(self.pop()?);
        }
        list.reverse();
        self.stack.push(Value::List(Rc::new(RefCell::new(list))));
        Ok(())
    }

    pub(super) fn exec_make_dict(&mut self, bytecode: &Bytecode, count: usize) -> Result<(), LugliError> {
        let mut dict = HashMap::new();
        for _ in 0..count {
            let value = self.pop()?;
            let key = self.pop()?;
            if let Value::String(key_str) = key {
                dict.insert(key_str, value);
            } else {
                return Err(LugliError::runtime("Dictionary keys must be strings"));
            }
        }

        // Check if this is a struct instantiation (has __struct_type__ field)
        let struct_type_key = bytecode.string_pool.borrow_mut().intern("__struct_type__");
        if let Some(Value::String(struct_type_id)) = dict.get(&struct_type_key) {
            let struct_type = self
                .resolve_string_id(*struct_type_id)
                .ok_or_else(|| LugliError::runtime(format!("Invalid struct type id: {}", struct_type_id.as_u32())))?;

            // Look up struct metadata from globals
            if let Some(Value::Dict(meta)) = self.globals.get(&struct_type) {
                let meta_ref = meta.borrow();

                // Get default values from struct metadata
                let defaults_key = bytecode.string_pool.borrow_mut().intern("__defaults__");
                if let Some(Value::Dict(defaults)) = meta_ref.get(&defaults_key) {
                    let defaults_ref = defaults.borrow();

                    // Apply defaults for missing fields
                    for (field_name, default_value) in defaults_ref.iter() {
                        if !dict.contains_key(field_name) {
                            dict.insert(*field_name, default_value.clone());
                        }
                    }
                }
            }
        }

        self.stack.push(Value::Dict(Rc::new(RefCell::new(dict))));
        Ok(())
    }

    pub(super) fn exec_define_function(&mut self, bytecode: &Bytecode, function_index: usize) -> Result<(), LugliError> {
        let function_value = self.get_constant(bytecode, function_index)?.clone();
        self.stack.push(function_value);
        Ok(())
    }

    pub(super) fn exec_make_closure(&mut self, bytecode: &Bytecode, function_index: usize, capture_indices: &[usize]) -> Result<(), LugliError> {
        // Get the base function template
        let function_value = self.get_constant(bytecode, function_index)?;

        if let Value::Function {
            name,
            params,
            body_start,
            bytecode_id,
        } = function_value
        {
            // Get current stack frame to calculate absolute positions
            let frame = self
                .call_stack
                .last()
                .ok_or_else(|| LugliError::runtime("Call stack is empty during closure creation"))?;

            // Capture values from the stack using open upvalues pattern
            let mut upvalues: Vec<Rc<RefCell<Value>>> = Vec::new();

            for &local_index in capture_indices {
                let absolute_index = frame.stack_base + local_index;

                // Check if this stack slot already has a shared reference
                let upvalue = if let Some(existing) = self.open_upvalues.get(&absolute_index) {
                    Rc::clone(existing)
                } else {
                    let value = self.stack.get(absolute_index).cloned().ok_or_else(|| {
                        LugliError::runtime(format!(
                            "Invalid upvalue capture: local {} (absolute {}) out of bounds (stack size: {})",
                            local_index, absolute_index, self.stack.len()
                        ))
                    })?;
                    let shared = Rc::new(RefCell::new(value));
                    self.open_upvalues.insert(absolute_index, Rc::clone(&shared));
                    shared
                };

                upvalues.push(upvalue);
            }

            // Create closure with captured upvalues
            let closure = Value::Closure {
                name: name.clone(),
                params: params.clone(),
                body_start: *body_start,
                bytecode_id: *bytecode_id,
                upvalues,
            };

            self.stack.push(closure);
            Ok(())
        } else {
            Err(LugliError::runtime("MakeClosure requires a Function value"))
        }
    }
}
