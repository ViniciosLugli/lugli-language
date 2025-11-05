use lugli_common::{LugliError, StringPool, Value};
use std::{cell::RefCell, rc::Rc};

pub fn list_length(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("list.len expects 1 argument"));
    }
    match &args[0] {
        Value::List(l) => Ok(Value::Number(l.borrow().len() as f64)),
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

pub fn list_push(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 2 {
        return Err(LugliError::runtime("list.push expects 2 arguments (list, item)"));
    }
    match &args[0] {
        Value::List(l) => {
            l.borrow_mut().push(args[1].clone());
            Ok(Value::Null)
        }
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

pub fn list_pop(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("list.pop expects 1 argument"));
    }
    match &args[0] {
        Value::List(l) => match l.borrow_mut().pop() {
            Some(val) => Ok(val),
            None => Ok(Value::Null),
        },
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

pub fn list_join(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.is_empty() || args.len() > 2 {
        return Err(LugliError::runtime("list.join expects 1 or 2 arguments"));
    }

    let separator = if args.len() == 2 {
        match &args[1] {
            Value::String(id) => pool.resolve(*id),
            _ => return Err(LugliError::type_error("string", args[1].type_name())),
        }
    } else {
        ""
    };

    match &args[0] {
        Value::List(l) => {
            let strings: Vec<String> = l.borrow().iter().map(|v| v.display_with_pool(pool)).collect();
            let joined = strings.join(separator);
            Ok(Value::String(pool.intern(&joined)))
        }
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

pub fn list_contains(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 2 {
        return Err(LugliError::runtime("list.contains expects 2 arguments (list, item)"));
    }
    match &args[0] {
        Value::List(l) => {
            let contains = l.borrow().iter().any(|v| v.equals(&args[1]));
            Ok(Value::Bool(contains))
        }
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

pub fn list_is_empty(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("list.is_empty expects 1 argument"));
    }
    match &args[0] {
        Value::List(l) => Ok(Value::Bool(l.borrow().is_empty())),
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

pub fn list_clear(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("list.clear expects 1 argument"));
    }
    match &args[0] {
        Value::List(l) => {
            l.borrow_mut().clear();
            Ok(Value::Null)
        }
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

pub fn list_reverse(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("list.reverse expects 1 argument"));
    }
    match &args[0] {
        Value::List(l) => {
            l.borrow_mut().reverse();
            Ok(Value::Null)
        }
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

pub fn list_append(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 2 {
        return Err(LugliError::runtime("list.append expects 2 arguments"));
    }
    match (&args[0], &args[1]) {
        (Value::List(l1), Value::List(l2)) => {
            if Rc::ptr_eq(l1, l2) {
                return Err(LugliError::runtime("Cannot append a list to itself (would create circular reference)"));
            }
            let items = l2.try_borrow().map_err(|_| LugliError::runtime("Cannot append list while it's being modified"))?.clone();
            l1.borrow_mut().extend(items);
            Ok(Value::Null)
        }
        (Value::List(_), _) => Err(LugliError::type_error("list", args[1].type_name())),
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

pub fn list_get(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 2 {
        return Err(LugliError::runtime("list.get expects 2 arguments (list, index)"));
    }
    match (&args[0], &args[1]) {
        (Value::List(l), Value::Number(idx)) => {
            if !idx.is_finite() {
                return Err(LugliError::runtime(format!("Index must be a finite number, got {}", idx)));
            }
            if idx.fract() != 0.0 {
                return Err(LugliError::runtime(format!("Index must be an integer, got {}", idx)));
            }
            let idx_i64 = *idx as i64;
            if idx_i64 < 0 {
                return Err(LugliError::runtime(format!("Negative index {} not allowed in list.get", idx)));
            }
            let index = idx_i64 as usize;
            let list = l.borrow();
            if index < list.len() { Ok(list[index].clone()) } else { Ok(Value::Null) }
        }
        (Value::List(_), _) => Err(LugliError::type_error("number", args[1].type_name())),
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

pub fn list_set(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 3 {
        return Err(LugliError::runtime("list.set expects 3 arguments (list, index, value)"));
    }
    match (&args[0], &args[1]) {
        (Value::List(l), Value::Number(idx)) => {
            if !idx.is_finite() {
                return Err(LugliError::runtime(format!("Index must be a finite number, got {}", idx)));
            }
            if idx.fract() != 0.0 {
                return Err(LugliError::runtime(format!("Index must be an integer, got {}", idx)));
            }
            let idx_i64 = *idx as i64;
            if idx_i64 < 0 {
                return Err(LugliError::runtime(format!("Negative index {} not allowed in list.set", idx)));
            }
            let index = idx_i64 as usize;
            let mut list = l.borrow_mut();
            if index < list.len() {
                list[index] = args[2].clone();
                Ok(Value::Null)
            } else {
                Err(LugliError::runtime(format!("Index {} out of bounds (list length {})", index, list.len())))
            }
        }
        (Value::List(_), _) => Err(LugliError::type_error("number", args[1].type_name())),
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

pub fn list_map(_args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    Err(LugliError::runtime(
        "map() is not yet implemented as a native function. Use list comprehensions instead:\n  \
         result = [transform(x) for x in list]",
    ))
}

pub fn list_filter(_args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    Err(LugliError::runtime(
        "filter() is not yet implemented as a native function. Use list comprehensions instead:\n  \
         result = [x for x in list if condition(x)]",
    ))
}

pub fn list_sorted(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.is_empty() || args.len() > 2 {
        return Err(LugliError::runtime("sorted expects 1 or 2 arguments"));
    }

    match &args[0] {
        Value::List(list) => {
            let reverse = if args.len() == 2 {
                match &args[1] {
                    Value::Bool(b) => *b,
                    _ => false,
                }
            } else {
                false
            };

            let mut items = {
                let borrowed = list.borrow();
                let mut new_items = Vec::with_capacity(borrowed.len());
                new_items.extend(borrowed.iter().cloned());
                new_items
            };

            items.sort_by(|a, b| match (a, b) {
                (Value::Number(x), Value::Number(y)) => {
                    if reverse {
                        y.partial_cmp(x).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                    }
                }
                (Value::String(x), Value::String(y)) => {
                    if reverse {
                        y.cmp(x)
                    } else {
                        x.cmp(y)
                    }
                }
                _ => std::cmp::Ordering::Equal,
            });

            Ok(Value::List(Rc::new(RefCell::new(items))))
        }
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

pub fn list_reversed(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("reversed expects 1 argument"));
    }

    match &args[0] {
        Value::List(list) => {
            let items = {
                let borrowed = list.borrow();
                let mut new_items = Vec::with_capacity(borrowed.len());
                new_items.extend(borrowed.iter().rev().cloned());
                new_items
            };
            Ok(Value::List(Rc::new(RefCell::new(items))))
        }
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}
