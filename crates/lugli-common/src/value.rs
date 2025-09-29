use chrono::{DateTime, TimeZone, Utc};
use hashbrown::HashMap;
use std::{
    cell::RefCell,
    fmt::{Debug, Formatter, Result as FmtResult},
    rc::Rc,
};

pub type NativeFunctionCallback = fn(&[Value]) -> Result<Value, crate::LugliError>;

#[derive(Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Null,
    Bool(bool),
    DateTime(DateTime<Utc>),
    List(Rc<RefCell<Vec<Value>>>),
    Dict(HashMap<String, Value>),
    Function {
        name: String,
        params: Vec<String>,
        arity: usize,
        body_start: usize,
    },
    NativeFunction {
        name: String,
        callback: NativeFunctionCallback,
        arity: usize
    },
    Struct {
        name: String,
        fields: HashMap<String, Value>
    },
    Constant(Box<Value>),
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::DateTime(dt) => write!(f, "{}", dt.to_rfc3339()),
            Value::List(items) => {
                let items = items.borrow();
                let formatted: Vec<String> = items.iter().map(|v| format!("{:?}", v)).collect();
                write!(f, "[{}]", formatted.join(", "))
            }
            Value::Dict(map) => {
                let formatted: Vec<String> = map.iter()
                    .map(|(k, v)| format!("{}: {:?}", k, v))
                    .collect();
                write!(f, "{{{}}}", formatted.join(", "))
            }
            Value::Function { name, params, .. } => {
                write!(f, "<function {}({})>", name, params.join(", "))
            }
            Value::NativeFunction { name, .. } => {
                write!(f, "<native {}>", name)
            }
            Value::Struct { name, fields } => {
                let field_names: Vec<String> = fields.keys().cloned().collect();
                write!(f, "<struct {} {{ {} }}>", name, field_names.join(", "))
            }
            Value::Constant(v) => write!(f, "const({:?})", v),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::DateTime(dt) => write!(f, "{}", dt.to_rfc3339()),
            Value::List(items) => {
                let items = items.borrow();
                let formatted: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", formatted.join(", "))
            }
            Value::Dict(map) => {
                let formatted: Vec<String> = map.iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                write!(f, "{{{}}}", formatted.join(", "))
            }
            _ => write!(f, "{:?}", self),
        }
    }
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Bool(_) => "bool",
            Value::Null => "null",
            Value::DateTime(_) => "datetime",
            Value::List(_) => "list",
            Value::Dict(_) => "dict",
            Value::Function { .. } => "function",
            Value::NativeFunction { .. } => "function",
            Value::Struct { .. } => "struct",
            Value::Constant(v) => v.type_name(),
        }
    }

    pub fn to_number(&self) -> f64 {
        match self {
            Value::Number(n) => *n,
            Value::Bool(true) => 1.0,
            Value::Bool(false) | Value::Null => 0.0,
            Value::String(s) => s.trim().parse().unwrap_or(0.0),
            Value::DateTime(dt) => dt.timestamp() as f64,
            Value::Constant(v) => v.to_number(),
            _ => 0.0,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => String::new(),
            Value::DateTime(dt) => dt.to_rfc3339(),
            Value::Constant(v) => v.to_string(),
            _ => format!("{}", self),
        }
    }

    pub fn to_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::List(l) => !l.borrow().is_empty(),
            Value::Dict(d) => !d.is_empty(),
            Value::Null => false,
            Value::DateTime(_) | Value::Function { .. } | Value::NativeFunction { .. } => true,
            Value::Struct { .. } => true,
            Value::Constant(v) => v.to_bool(),
        }
    }

    pub fn to_datetime(&self) -> Result<DateTime<Utc>, crate::LugliError> {
        match self {
            Value::DateTime(dt) => Ok(*dt),
            Value::Number(n) => Ok(Utc.timestamp_opt(*n as i64, 0).single()
                .ok_or_else(|| crate::LugliError::runtime("Invalid timestamp"))?),
            Value::String(s) => {
                DateTime::parse_from_rfc3339(s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .or_else(|_| {
                        DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                            .map(|dt| dt.with_timezone(&Utc))
                    })
                    .map_err(|_| crate::LugliError::runtime("Invalid datetime format"))
            }
            Value::Constant(v) => v.to_datetime(),
            _ => Err(crate::LugliError::type_error("datetime", self.type_name())),
        }
    }

    pub fn is_truthy(&self) -> bool {
        self.to_bool()
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::DateTime(a), Value::DateTime(b)) => a == b,
            (Value::Constant(a), b) => a.equals(b),
            (a, Value::Constant(b)) => a.equals(b),
            _ => false,
        }
    }
}