use super::Machine;
use crate::Bytecode;
use lugli_common::{LugliError, Value};

// Utility and module operation instruction handlers
impl Machine {
    pub(super) fn exec_to_string(&mut self, bytecode: &Bytecode) -> Result<(), LugliError> {
        let value = self.pop()?;
        let pool = bytecode.string_pool.borrow();
        let string_value = match &value {
            Value::String(s) => pool.resolve(*s).to_string(),
            Value::Number(n) => {
                if n.is_nan() {
                    "NaN".to_string()
                } else if n.is_infinite() {
                    if n.is_sign_positive() { "Infinity".to_string() } else { "-Infinity".to_string() }
                } else if n.fract() == 0.0 && n.abs() < 1e15 {
                    format!("{:.0}", n)
                } else {
                    n.to_string()
                }
            }
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::List(list) => {
                let list_ref = list.borrow();
                let elements: Vec<String> = list_ref
                    .iter()
                    .map(|v| match v {
                        Value::String(s) => format!("\"{}\"", pool.resolve(*s)),
                        _ => v.display_with_pool(&pool),
                    })
                    .collect();
                format!("[{}]", elements.join(", "))
            }
            Value::Dict(dict) => {
                let dict_ref = dict.borrow();
                let pairs: Vec<String> = dict_ref
                    .iter()
                    .map(|(k, v)| match v {
                        Value::String(s) => format!("\"{}\": \"{}\"", pool.resolve(*k), pool.resolve(*s)),
                        _ => format!("\"{}\": {}", pool.resolve(*k), v.display_with_pool(&pool)),
                    })
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            Value::Function {
                name, ..
            }
            | Value::Closure {
                name, ..
            } => format!("<function {}>", name),
            Value::NativeFunction {
                name, ..
            } => format!("<native function {}>", name),
            Value::StructInstance {
                name, ..
            } => format!("<{} instance>", name),
            Value::DateTime(dt) => dt.to_string(),
            Value::Module {
                path, ..
            } => format!("<module {}>", path),
        };
        drop(pool);
        let id = bytecode.string_pool.borrow_mut().intern(&string_value);
        self.stack.push(Value::String(id));
        Ok(())
    }

    pub(super) fn exec_import_module(&mut self, bytecode: &Bytecode, module_idx: usize, bind_name: &str) -> Result<(), LugliError> {
        let module_path = match &bytecode.constants[module_idx] {
            Value::String(s) => bytecode.string_pool.borrow().resolve(*s).to_string(),
            _ => return Err(LugliError::runtime("ImportModule: expected string constant")),
        };

        let exports = self.load_module(&module_path)?;

        let module_value = Value::Module {
            path: module_path,
            exports,
        };

        // Store directly in globals - no stack effect
        self.globals.insert(bind_name.to_string(), module_value);
        Ok(())
    }

    pub(super) fn exec_import_from(&mut self, bytecode: &Bytecode, module_idx: usize, names: &[String]) -> Result<(), LugliError> {
        let module_path = match &bytecode.constants[module_idx] {
            Value::String(s) => bytecode.string_pool.borrow().resolve(*s).to_string(),
            _ => return Err(LugliError::runtime("ImportFrom: expected string constant")),
        };

        let exports = self.load_module(&module_path)?;

        for name in names {
            if let Some(value) = exports.get(name) {
                self.globals.insert(name.clone(), value.clone());
            } else {
                return Err(LugliError::runtime(format!("Module '{}' has no export '{}'", module_path, name)));
            }
        }
        Ok(())
    }
}
