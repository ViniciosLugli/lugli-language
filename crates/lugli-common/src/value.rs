use crate::{
    error::LugliError,
    string_pool::{StringId, StringPool},
};
use hashbrown::HashMap;
use std::{
    cell::RefCell,
    collections::hash_map::DefaultHasher,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    rc::Rc,
};

pub type NativeFunction = fn(&[Value], &mut StringPool) -> Result<Value, LugliError>;

/// Runtime value representation for Lugli language.
///
/// # Memory Management & Circular References
///
/// **WARNING**: Circular references in reference-counted types (Dict, List, Closure, StructInstance)
/// will cause memory leaks. Rust's `Rc<RefCell<>>` uses reference counting without cycle detection.
///
/// ## Example of Memory Leak:
/// ```lugli
/// let x = {}
/// x.self = x  # Creates cycle: x → Dict → x
/// # When x goes out of scope, Rc count never reaches 0 → memory leak
/// ```
///
/// ## Mitigation Strategies:
/// 1. **Avoid self-references**: Don't assign objects to themselves or create cycles
/// 2. **Use GC monitoring**: The `GarbageCollector` tracks allocations (see `lugli-common::gc`)
/// 3. **Break cycles manually**: Set fields to `null` before dropping large structures
/// 4. **Weak references**: Future versions will support `WeakDict` and `WeakList` variants
///
/// ## Similar to:
/// - **Python**: Uses refcounting + cycle detection (we only have refcounting)
/// - **Ruby**: Uses tri-color marking GC (we have basic mark-sweep)
/// - **Lua**: Uses incremental GC (our GC is adaptive threshold-based)
///
/// ## Production Usage:
/// For long-running applications, monitor memory usage and avoid circular data structures.
/// Use the VM's GC statistics (`GarbageCollector::stats()`) to tune thresholds.
#[derive(Clone)]
pub enum Value {
    Number(f64),
    String(StringId),
    Bool(bool),
    Null,
    /// List with potential for circular references (see type docs)
    List(Rc<RefCell<Vec<Value>>>),
    /// Dictionary with potential for circular references (see type docs)
    Dict(Rc<RefCell<HashMap<StringId, Value>>>),
    Function {
        name: Rc<str>,
        params: Rc<Vec<String>>,
        body_start: usize,
        bytecode_id: usize,
        required_count: usize,
        defaults: Rc<Vec<Value>>,
    },
    Closure {
        name: Rc<str>,
        params: Rc<Vec<String>>,
        body_start: usize,
        bytecode_id: usize,
        upvalues: Vec<Rc<RefCell<Value>>>,
        required_count: usize,
        defaults: Rc<Vec<Value>>,
    },
    NativeFunction {
        name: String,
        callback: NativeFunction,
        arity: usize,
    },
    StructInstance {
        name: String,
        fields: HashMap<StringId, Value>,
    },
    DateTime(i64),
    Module {
        path: String,
        exports: HashMap<String, Value>,
    },
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool { self.equals(other) }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);

        match self {
            Value::Number(n) => n.to_bits().hash(state),
            Value::String(id) => id.hash(state),
            Value::Bool(b) => b.hash(state),
            Value::Null => {}
            Value::List(list) => {
                if let Ok(list_ref) = list.try_borrow() {
                    list_ref.len().hash(state);
                    for item in list_ref.iter() {
                        item.hash(state);
                    }
                }
            }
            Value::Dict(dict) => {
                if let Ok(dict_ref) = dict.try_borrow() {
                    let mut hash_acc = 0u64;
                    for (k, v) in dict_ref.iter() {
                        let mut hasher = DefaultHasher::new();
                        k.hash(&mut hasher);
                        v.hash(&mut hasher);
                        hash_acc ^= hasher.finish();
                    }
                    hash_acc.hash(state);
                }
            }
            Value::Function {
                bytecode_id, ..
            } => bytecode_id.hash(state),
            Value::Closure {
                bytecode_id,
                upvalues,
                ..
            } => {
                bytecode_id.hash(state);
                upvalues.len().hash(state);
                for uv in upvalues {
                    if let Ok(uv_ref) = uv.try_borrow() {
                        uv_ref.hash(state);
                    }
                }
            }
            Value::NativeFunction {
                name, ..
            } => name.hash(state),
            Value::StructInstance {
                name, ..
            } => name.hash(state),
            Value::DateTime(ts) => ts.hash(state),
            Value::Module {
                path, ..
            } => path.hash(state),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(id) => write!(f, "<string#{}>", id.as_u32()),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::List(l) => match l.try_borrow() {
                Ok(list_ref) => {
                    let items: Vec<String> = list_ref.iter().map(|v| v.to_string()).collect();
                    write!(f, "[{}]", items.join(", "))
                }
                Err(_) => write!(f, "[<borrowed list>]"),
            },
            Value::Dict(d) => match d.try_borrow() {
                Ok(dict_ref) => {
                    let items: Vec<String> = dict_ref.iter().map(|(k, v)| format!("<string#{}>: {}", k.as_u32(), v)).collect();
                    write!(f, "{{ {} }}", items.join(", "))
                }
                Err(_) => write!(f, "{{<borrowed dict>}}"),
            },
            Value::Function {
                name, ..
            } => write!(f, "<fn {}>", name),
            Value::Closure {
                name, ..
            } => write!(f, "<closure {}>", name),
            Value::NativeFunction {
                name, ..
            } => write!(f, "<native fn {}>", name),
            Value::StructInstance {
                name, ..
            } => write!(f, "<struct {} instance>", name),
            Value::DateTime(ts) => write!(f, "<datetime {}>", ts),
            Value::Module {
                path, ..
            } => write!(f, "<module {}>", path),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Value::String(id) => write!(f, "String(#{})", id.as_u32()),
            Value::List(l) => match l.try_borrow() {
                Ok(list_ref) => {
                    let items: Vec<String> = list_ref.iter().map(|v| format!("{:?}", v)).collect();
                    write!(f, "[{}]", items.join(", "))
                }
                Err(_) => write!(f, "[<borrowed list>]"),
            },
            Value::Dict(d) => match d.try_borrow() {
                Ok(dict_ref) => {
                    let items: Vec<String> = dict_ref.iter().map(|(k, v)| format!("<string#{}>: {:?}", k.as_u32(), v)).collect();
                    write!(f, "{{ {} }}", items.join(", "))
                }
                Err(_) => write!(f, "{{<borrowed dict>}}"),
            },
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
            Value::Closure {
                name,
                params,
                body_start,
                bytecode_id,
                upvalues,
                required_count,
                defaults,
            } => Value::Closure {
                name: name.clone(),
                params: params.clone(),
                body_start: *body_start,
                bytecode_id: *bytecode_id,
                upvalues: upvalues.iter().map(Rc::clone).collect(),
                required_count: *required_count,
                defaults: defaults.clone(),
            },
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
            Value::Function {
                ..
            }
            | Value::Closure {
                ..
            }
            | Value::NativeFunction {
                ..
            } => "function",
            Value::StructInstance {
                ..
            } => "struct",
            Value::DateTime(_) => "datetime",
            Value::Module {
                ..
            } => "module",
        }
    }

    pub fn type_id(&self) -> usize {
        match self {
            Value::Number(_) => 0,
            Value::String(_) => 1,
            Value::Bool(_) => 2,
            Value::Null => 3,
            Value::List(_) => 4,
            Value::Dict(_) => 5,
            Value::Function {
                ..
            } => 6,
            Value::Closure {
                ..
            } => 7,
            Value::NativeFunction {
                ..
            } => 8,
            Value::StructInstance {
                ..
            } => 9,
            Value::DateTime(_) => 10,
            Value::Module {
                ..
            } => 11,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Number(n) => *n != 0.0,
            Value::String(_) => true,
            _ => true,
        }
    }

    pub fn is_truthy_with_pool(&self, pool: &StringPool) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Number(n) => *n != 0.0,
            Value::String(id) => !pool.resolve(*id).is_empty(),
            _ => true,
        }
    }

    /// Check if this value contains circular references
    ///
    /// Detects both direct (x.self = x) and indirect (a→b→a) cycles
    /// Uses depth-first search with visited set tracking
    pub fn contains_cycle(&self) -> bool {
        use std::collections::HashSet;
        let mut visited = HashSet::new();
        self.contains_cycle_impl(&mut visited)
    }

    fn contains_cycle_impl(&self, visited: &mut std::collections::HashSet<usize>) -> bool {
        match self {
            Value::Dict(d) => {
                let ptr = Rc::as_ptr(d) as usize;
                if visited.contains(&ptr) {
                    return true; // Cycle detected
                }

                visited.insert(ptr);

                if let Ok(dict_ref) = d.try_borrow() {
                    for v in dict_ref.values() {
                        if v.contains_cycle_impl(visited) {
                            visited.remove(&ptr);
                            return true;
                        }
                    }
                }

                visited.remove(&ptr);
                false
            }
            Value::List(l) => {
                let ptr = Rc::as_ptr(l) as usize;
                if visited.contains(&ptr) {
                    return true; // Cycle detected
                }

                visited.insert(ptr);

                if let Ok(list_ref) = l.try_borrow() {
                    for v in list_ref.iter() {
                        if v.contains_cycle_impl(visited) {
                            visited.remove(&ptr);
                            return true;
                        }
                    }
                }

                visited.remove(&ptr);
                false
            }
            Value::Closure {
                upvalues, ..
            } => {
                // Check upvalues for cycles
                for upvalue in upvalues {
                    if let Ok(uv_ref) = upvalue.try_borrow()
                        && uv_ref.contains_cycle_impl(visited)
                    {
                        return true;
                    }
                }
                false
            }
            Value::StructInstance {
                fields, ..
            } => {
                // Use stable hash of fields HashMap pointer
                let ptr = (fields as *const HashMap<StringId, Value>) as usize;
                if visited.contains(&ptr) {
                    return true;
                }

                visited.insert(ptr);

                for v in fields.values() {
                    if v.contains_cycle_impl(visited) {
                        visited.remove(&ptr);
                        return true;
                    }
                }

                visited.remove(&ptr);
                false
            }
            // Primitives and functions cannot form cycles
            _ => false,
        }
    }

    pub fn display_with_pool(&self, pool: &StringPool) -> String {
        match self {
            Value::Number(n) => n.to_string(),
            Value::String(id) => pool.resolve(*id).to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::List(l) => match l.try_borrow() {
                Ok(list_ref) => {
                    let items: Vec<String> = list_ref.iter().map(|v| v.display_with_pool(pool)).collect();
                    format!("[{}]", items.join(", "))
                }
                Err(_) => "[<borrowed list>]".to_string(),
            },
            Value::Dict(d) => match d.try_borrow() {
                Ok(dict_ref) => {
                    let items: Vec<String> =
                        dict_ref.iter().map(|(k, v)| format!("\"{}\": {}", pool.resolve(*k), v.display_with_pool(pool))).collect();
                    format!("{{ {} }}", items.join(", "))
                }
                Err(_) => "{{<borrowed dict>}}".to_string(),
            },
            Value::Function {
                name, ..
            } => format!("<fn {}>", name),
            Value::Closure {
                name, ..
            } => format!("<closure {}>", name),
            Value::NativeFunction {
                name, ..
            } => format!("<native fn {}>", name),
            Value::StructInstance {
                name, ..
            } => format!("<struct {} instance>", name),
            Value::DateTime(ts) => format!("<datetime {}>", ts),
            Value::Module {
                path, ..
            } => format!("<module {}>", path),
        }
    }

    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::List(a), Value::List(b)) => Rc::ptr_eq(a, b),
            (Value::Dict(a), Value::Dict(b)) => match (a.try_borrow(), b.try_borrow()) {
                (Ok(a_ref), Ok(b_ref)) => {
                    if a_ref.len() != b_ref.len() {
                        return false;
                    }
                    a_ref.iter().all(|(k, v)| b_ref.get(k).is_some_and(|v2| v.equals(v2)))
                }
                _ => false,
            },
            (
                Value::Function {
                    name: n1,
                    params: p1,
                    body_start: b1,
                    bytecode_id: id1,
                    ..
                },
                Value::Function {
                    name: n2,
                    params: p2,
                    body_start: b2,
                    bytecode_id: id2,
                    ..
                },
            ) => n1 == n2 && p1 == p2 && b1 == b2 && id1 == id2,
            (
                Value::Closure {
                    name: n1,
                    params: p1,
                    body_start: b1,
                    bytecode_id: id1,
                    upvalues: u1,
                    ..
                },
                Value::Closure {
                    name: n2,
                    params: p2,
                    body_start: b2,
                    bytecode_id: id2,
                    upvalues: u2,
                    ..
                },
            ) => {
                if n1 != n2 || p1 != p2 || b1 != b2 || id1 != id2 || u1.len() != u2.len() {
                    return false;
                }
                u1.iter().zip(u2.iter()).all(|(uv_a, uv_b)| match (uv_a.try_borrow(), uv_b.try_borrow()) {
                    (Ok(a_ref), Ok(b_ref)) => a_ref.equals(&b_ref),
                    _ => false,
                })
            }
            (
                Value::NativeFunction {
                    name: n1, ..
                },
                Value::NativeFunction {
                    name: n2, ..
                },
            ) => n1 == n2,
            (
                Value::StructInstance {
                    name: n1,
                    fields: f1,
                },
                Value::StructInstance {
                    name: n2,
                    fields: f2,
                },
            ) => n1 == n2 && f1 == f2,
            (Value::DateTime(a), Value::DateTime(b)) => a == b,
            (
                Value::Module {
                    path: p1, ..
                },
                Value::Module {
                    path: p2, ..
                },
            ) => p1 == p2,
            _ => false,
        }
    }

    // Operator implementations
    pub fn add(&self, other: &Value) -> Result<Value, LugliError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(_), Value::String(_)) => Err(LugliError::runtime("String concatenation requires string pool (use add_with_pool)")),
            (Value::List(a), Value::List(b)) => {
                let mut result = a.borrow().clone();
                result.extend(b.borrow().iter().map(|v| v.clone_for_stack()));
                Ok(Value::List(Rc::new(RefCell::new(result))))
            }
            _ => Err(LugliError::runtime(format!("Unsupported operand types for +: {} and {}", self.type_name(), other.type_name()))),
        }
    }

    pub fn add_with_pool(&self, other: &Value, pool: &mut StringPool) -> Result<Value, LugliError> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => {
                let concat = format!("{}{}", pool.resolve(*a), pool.resolve(*b));
                Ok(Value::String(pool.intern(&concat)))
            }
            (Value::List(a), Value::List(b)) => {
                let mut result = a.borrow().clone();
                result.extend(b.borrow().iter().map(|v| v.clone_for_stack()));
                Ok(Value::List(Rc::new(RefCell::new(result))))
            }
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
