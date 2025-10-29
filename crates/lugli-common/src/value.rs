use crate::error::LugliError;
use hashbrown::HashMap;
use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::rc::Rc;

pub type NativeFunction = fn(&[Value]) -> Result<Value, LugliError>;

#[derive(Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    List(Rc<RefCell<Vec<Value>>>),
    Dict(Rc<RefCell<HashMap<String, Value>>>),
    Function {
        name: String,
        params: Vec<String>,
        body_start: usize,
    },
    Closure {
        name: String,
        params: Vec<String>,
        body_start: usize,
        upvalues: Rc<RefCell<Vec<Value>>>,
    },
    NativeFunction {
        name: String,
        callback: NativeFunction,
        arity: usize,
    },
    StructInstance { name: String, fields: HashMap<String, Value> },
    DateTime(i64), // Unix timestamp
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::List(a), Value::List(b)) => Rc::ptr_eq(a, b),
            (Value::Dict(a), Value::Dict(b)) => Rc::ptr_eq(a, b),
            (Value::Function { name: n1, params: p1, body_start: b1 },
             Value::Function { name: n2, params: p2, body_start: b2 }) => {
                n1 == n2 && p1 == p2 && b1 == b2
            }
            (Value::Closure { name: n1, params: p1, body_start: b1, upvalues: u1 },
             Value::Closure { name: n2, params: p2, body_start: b2, upvalues: u2 }) => {
                n1 == n2 && p1 == p2 && b1 == b2 && Rc::ptr_eq(u1, u2)
            }
            (Value::NativeFunction { name: n1, .. },
             Value::NativeFunction { name: n2, .. }) => n1 == n2,
            (Value::StructInstance { name: n1, fields: f1 },
             Value::StructInstance { name: n2, fields: f2 }) => {
                n1 == n2 && f1 == f2
            }
            (Value::DateTime(a), Value::DateTime(b)) => a == b,
            _ => false,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::List(l) => {
                let items: Vec<String> = l.borrow().iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Value::Dict(d) => {
                let items: Vec<String> = d.borrow().iter().map(|(k, v)| format!("\"{}\": {}", k, v)).collect();
                write!(f, "{{ {} }}", items.join(", "))
            }
            Value::Function { name, .. } => write!(f, "<fn {}>", name),
            Value::Closure { name, .. } => write!(f, "<closure {}>", name),
            Value::NativeFunction { name, .. } => write!(f, "<native fn {}>", name),
            Value::StructInstance { name, .. } => write!(f, "<struct {} instance>", name),
            Value::DateTime(ts) => write!(f, "<datetime {}>", ts),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::List(l) => write!(f, "[{}]", l.borrow().iter().map(|v| format!("{:?}", v)).collect::<Vec<String>>().join(", ")),
            Value::Dict(d) => {
                let items: Vec<String> = d.borrow().iter().map(|(k, v)| format!("\"{}\": {:?}", k, v)).collect();
                write!(f, "{{ {} }}", items.join(", "))
            }
            _ => write!(f, "{}", self),
        }
    }
}

impl Value {
    /// Optimized clone for stack operations
    /// Copy types (Number, Bool, Null, DateTime) are copied directly
    /// Reference types (List, Dict) increment Rc counter (cheap)
    /// Only String and complex types actually allocate
    #[inline]
    pub fn clone_for_stack(&self) -> Self {
        match self {
            Value::Number(n) => Value::Number(*n),
            Value::Bool(b) => Value::Bool(*b),
            Value::Null => Value::Null,
            Value::DateTime(ts) => Value::DateTime(*ts),
            // Reference types use Rc::clone (cheap pointer copy)
            Value::List(list) => Value::List(Rc::clone(list)),
            Value::Dict(dict) => Value::Dict(Rc::clone(dict)),
            Value::Closure { upvalues, .. } => {
                // For closures, only clone upvalues Rc
                match self {
                    Value::Closure { name, params, body_start, upvalues } => {
                        Value::Closure {
                            name: name.clone(),
                            params: params.clone(),
                            body_start: *body_start,
                            upvalues: Rc::clone(upvalues),
                        }
                    }
                    _ => unreachable!(),
                }
            }
            // Other types need full clone (String allocates)
            _ => self.clone(),
        }
    }


    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Bool(_) => "bool",
            Value::Null => "null",
            Value::List(_) => "list",
            Value::Dict(_) => "dict",
            Value::Function { .. } | Value::Closure { .. } | Value::NativeFunction { .. } => "function",
            Value::StructInstance { .. } => "struct",
            Value::DateTime(_) => "datetime",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            _ => true, // All other types are truthy
        }
    }

    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::List(a), Value::List(b)) => Rc::ptr_eq(a, b),
            (Value::Dict(a), Value::Dict(b)) => Rc::ptr_eq(a, b),
            (Value::Function { name: n1, params: p1, body_start: b1 },
             Value::Function { name: n2, params: p2, body_start: b2 }) => {
                n1 == n2 && p1 == p2 && b1 == b2
            }
            (Value::NativeFunction { name: n1, .. },
             Value::NativeFunction { name: n2, .. }) => n1 == n2,
            (Value::StructInstance { name: n1, fields: f1 },
             Value::StructInstance { name: n2, fields: f2 }) => {
                n1 == n2 && f1 == f2
            }
            (Value::DateTime(a), Value::DateTime(b)) => a == b,
            _ => false,
        }
    }

    // Operator implementations
    pub fn add(&self, other: &Value) -> Result<Value, LugliError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(LugliError::runtime(format!("Unsupported operand types for +: {} and {}", self.type_name(), other.type_name()))),
        }
    }

    pub fn subtract(&self, other: &Value) -> Result<Value, LugliError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            _ => Err(LugliError::runtime(format!("Unsupported operand types for -: {} and {}", self.type_name(), other.type_name()))),
        }
    }

    pub fn multiply(&self, other: &Value) -> Result<Value, LugliError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            _ => Err(LugliError::runtime(format!("Unsupported operand types for *: {} and {}", self.type_name(), other.type_name()))),
        }
    }

    pub fn divide(&self, other: &Value) -> Result<Value, LugliError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => {
                if *b == 0.0 {
                    Err(LugliError::runtime("Division by zero"))
                } else {
                    Ok(Value::Number(a / b))
                }
            }
            _ => Err(LugliError::runtime(format!("Unsupported operand types for /: {} and {}", self.type_name(), other.type_name()))),
        }
    }

    pub fn integer_divide(&self, other: &Value) -> Result<Value, LugliError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => {
                if *b == 0.0 {
                    Err(LugliError::runtime("Division by zero"))
                } else {
                    Ok(Value::Number((a / b).floor()))
                }
            }
            _ => Err(LugliError::runtime(format!("Unsupported operand types for //: {} and {}", self.type_name(), other.type_name()))),
        }
    }

    pub fn modulo(&self, other: &Value) -> Result<Value, LugliError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => {
                if *b == 0.0 {
                    Err(LugliError::runtime("Modulo by zero"))
                } else {
                    Ok(Value::Number(a % b))
                }
            }
            _ => Err(LugliError::runtime(format!("Unsupported operand types for %: {} and {}", self.type_name(), other.type_name()))),
        }
    }

    pub fn power(&self, other: &Value) -> Result<Value, LugliError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.powf(*b))),
            _ => Err(LugliError::runtime(format!("Unsupported operand types for **: {} and {}", self.type_name(), other.type_name()))),
        }
    }

    pub fn negate(&self) -> Result<Value, LugliError> {
        match self {
            Value::Number(n) => Ok(Value::Number(-n)),
            _ => Err(LugliError::runtime(format!("Unsupported operand type for -: {}", self.type_name()))),
        }
    }

    pub fn greater(&self, other: &Value) -> Result<Value, LugliError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a > b)),
            _ => Err(LugliError::runtime(format!("Unsupported operand types for >: {} and {}", self.type_name(), other.type_name()))),
        }
    }

    pub fn greater_equal(&self, other: &Value) -> Result<Value, LugliError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a >= b)),
            _ => Err(LugliError::runtime(format!("Unsupported operand types for >=: {} and {}", self.type_name(), other.type_name()))),
        }
    }

    pub fn less(&self, other: &Value) -> Result<Value, LugliError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a < b)),
            _ => Err(LugliError::runtime(format!("Unsupported operand types for <: {} and {}", self.type_name(), other.type_name()))),
        }
    }

    pub fn less_equal(&self, other: &Value) -> Result<Value, LugliError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a <= b)),
            _ => Err(LugliError::runtime(format!("Unsupported operand types for <=: {} and {}", self.type_name(), other.type_name()))),
        }
    }
}