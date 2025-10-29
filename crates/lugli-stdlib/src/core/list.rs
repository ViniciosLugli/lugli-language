use lugli_common::{LugliError, Value};

pub fn list_length(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("list.len expects 1 argument"));
    }
    match &args[0] {
        Value::List(l) => Ok(Value::Number(l.borrow().len() as f64)),
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

pub fn list_push(args: &[Value]) -> Result<Value, LugliError> {
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

pub fn list_pop(args: &[Value]) -> Result<Value, LugliError> {
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

pub fn list_join(args: &[Value]) -> Result<Value, LugliError> {
    if args.is_empty() || args.len() > 2 {
        return Err(LugliError::runtime("list.join expects 1 or 2 arguments"));
    }

    let separator = if args.len() == 2 {
        match &args[1] {
            Value::String(s) => s.as_str(),
            _ => return Err(LugliError::type_error("string", args[1].type_name())),
        }
    } else {
        ""
    };

    match &args[0] {
        Value::List(l) => {
            let strings: Vec<String> = l.borrow().iter().map(|v| v.to_string()).collect();
            Ok(Value::String(strings.join(separator)))
        }
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

pub fn list_contains(args: &[Value]) -> Result<Value, LugliError> {
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

pub fn list_is_empty(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("list.is_empty expects 1 argument"));
    }
    match &args[0] {
        Value::List(l) => Ok(Value::Bool(l.borrow().is_empty())),
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

pub fn list_clear(args: &[Value]) -> Result<Value, LugliError> {
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

pub fn list_reverse(args: &[Value]) -> Result<Value, LugliError> {
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

pub fn list_append(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 2 {
        return Err(LugliError::runtime("list.append expects 2 arguments"));
    }
    match (&args[0], &args[1]) {
        (Value::List(l1), Value::List(l2)) => {
            let items = l2.try_borrow().map_err(|_| LugliError::runtime("Cannot append list while it's being modified"))?.clone();
            l1.borrow_mut().extend(items);
            Ok(Value::Null)
        }
        (Value::List(_), _) => Err(LugliError::type_error("list", args[1].type_name())),
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

pub fn list_get(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 2 {
        return Err(LugliError::runtime("list.get expects 2 arguments (list, index)"));
    }
    match (&args[0], &args[1]) {
        (Value::List(l), Value::Number(idx)) => {
            let index = *idx as usize;
            let list = l.borrow();
            if index < list.len() { Ok(list[index].clone()) } else { Ok(Value::Null) }
        }
        (Value::List(_), _) => Err(LugliError::type_error("number", args[1].type_name())),
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

pub fn list_set(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 3 {
        return Err(LugliError::runtime("list.set expects 3 arguments (list, index, value)"));
    }
    match (&args[0], &args[1]) {
        (Value::List(l), Value::Number(idx)) => {
            let index = *idx as usize;
            let mut list = l.borrow_mut();
            if index < list.len() {
                list[index] = args[2].clone();
                Ok(Value::Null)
            } else {
                Err(LugliError::runtime(format!("Index {} out of bounds", index)))
            }
        }
        (Value::List(_), _) => Err(LugliError::type_error("number", args[1].type_name())),
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

// Higher-order functions
use std::{cell::RefCell, rc::Rc};

pub fn list_map(_args: &[Value]) -> Result<Value, LugliError> {
    // NOTE: map() requires VM context to execute callback functions
    // This functionality is currently implemented via list comprehensions:
    //   mapped = [transform(x) for x in list]
    //
    // Same limitation as filter() - requires VM integration
    Err(LugliError::runtime(
        "map() is not yet implemented as a native function. Use list comprehensions instead:\n  \
         result = [transform(x) for x in list]",
    ))
}

pub fn list_filter(_args: &[Value]) -> Result<Value, LugliError> {
    // NOTE: filter() requires VM context to execute callback functions
    // This functionality is currently implemented via list comprehensions:
    //   filtered = [x for x in list if predicate(x)]
    //
    // To implement as a native function would require:
    // 1. Passing VM/Machine context to stdlib functions
    // 2. Calling user-defined functions from native code
    // 3. Handling errors and stack management
    //
    // This is a future enhancement - use list comprehensions for now
    Err(LugliError::runtime(
        "filter() is not yet implemented as a native function. Use list comprehensions instead:\n  \
         result = [x for x in list if condition(x)]",
    ))
}

pub fn list_sorted(args: &[Value]) -> Result<Value, LugliError> {
    if args.is_empty() || args.len() > 2 {
        return Err(LugliError::runtime("sorted expects 1 or 2 arguments"));
    }

    match &args[0] {
        Value::List(list) => {
            let mut items = list.borrow().clone();
            let reverse = if args.len() == 2 {
                match &args[1] {
                    Value::Bool(b) => *b,
                    _ => false,
                }
            } else {
                false
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

pub fn list_reversed(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("reversed expects 1 argument"));
    }

    match &args[0] {
        Value::List(list) => {
            let mut items = list.borrow().clone();
            items.reverse();
            Ok(Value::List(Rc::new(RefCell::new(items))))
        }
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}
