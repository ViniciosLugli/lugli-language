use crate::NativeFunction;
use lugli_common::{LugliError, Value};
use std::{cell::RefCell, rc::Rc};

pub mod dict;
pub mod list;
pub mod number;
pub mod string;

pub fn get_functions() -> Vec<(&'static str, NativeFunction)> {
    let mut functions = vec![
        ("type", type_of as NativeFunction),
        ("len", len_fn as NativeFunction),
        ("range", range_fn as NativeFunction),
        ("enumerate", enumerate_fn as NativeFunction),
        ("str", str_fn as NativeFunction),
        ("int", int_fn as NativeFunction),
        ("float", float_fn as NativeFunction),
        ("bool", bool_fn as NativeFunction),
        // Collection utilities
        ("zip", zip_fn as NativeFunction),
        ("all", all_fn as NativeFunction),
        ("any", any_fn as NativeFunction),
        ("sum", sum_fn as NativeFunction),
        ("min", min_fn as NativeFunction),
        ("max", max_fn as NativeFunction),
        ("sorted", list::list_sorted as NativeFunction),
        ("reversed", list::list_reversed as NativeFunction),
        // Math functions
        ("abs", abs_fn as NativeFunction),
        ("round", round_fn as NativeFunction),
        ("pow", pow_fn as NativeFunction),
        // Higher-order functions
        ("map", map_fn as NativeFunction),
        ("filter", filter_fn as NativeFunction),
        ("reduce", reduce_fn as NativeFunction),
    ];

    // Add string methods - these will be accessed as regular functions for now
    // In the future, we'll handle method syntax (obj.method())
    functions.extend(vec![
        ("str_len", string::string_length as NativeFunction),
        ("str_trim", string::string_trim as NativeFunction),
        ("str_lower", string::string_lower as NativeFunction),
        ("str_upper", string::string_upper as NativeFunction),
        ("str_chars", string::string_chars as NativeFunction),
        ("str_split", string::string_split as NativeFunction),
        ("str_is_alphabetic", string::string_is_alphabetic as NativeFunction),
        ("str_starts_with", string::string_starts_with as NativeFunction),
        ("str_ends_with", string::string_ends_with as NativeFunction),
        ("str_contains", string::string_contains as NativeFunction),
        ("str_replace", string::string_replace as NativeFunction),
    ]);

    // Add list methods
    functions.extend(vec![
        ("list_len", list::list_length as NativeFunction),
        ("list_push", list::list_push as NativeFunction),
        ("list_pop", list::list_pop as NativeFunction),
        ("list_join", list::list_join as NativeFunction),
        ("list_contains", list::list_contains as NativeFunction),
        ("list_is_empty", list::list_is_empty as NativeFunction),
        ("list_clear", list::list_clear as NativeFunction),
        ("list_reverse", list::list_reverse as NativeFunction),
        ("list_append", list::list_append as NativeFunction),
        ("list_get", list::list_get as NativeFunction),
        ("list_set", list::list_set as NativeFunction),
    ]);

    functions
}

fn type_of(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("type expects 1 argument"));
    }
    Ok(Value::String(args[0].type_name().to_string()))
}

fn len_fn(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("len expects 1 argument"));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Number(s.len() as f64)),
        Value::List(l) => Ok(Value::Number(l.borrow().len() as f64)),
        Value::Dict(d) => Ok(Value::Number(d.borrow().len() as f64)),
        _ => Err(LugliError::runtime(format!("Object of type {} has no len()", args[0].type_name()))),
    }
}

fn range_fn(args: &[Value]) -> Result<Value, LugliError> {
    let (start, stop, step) = match args.len() {
        1 => {
            // range(n) -> range from 0 to n-1
            if let Value::Number(n) = args[0] {
                (0.0, n, 1.0)
            } else {
                return Err(LugliError::runtime("range expects numeric arguments"));
            }
        }
        2 => {
            // range(start, stop) -> range from start to stop-1
            if let (Value::Number(s), Value::Number(e)) = (&args[0], &args[1]) {
                (*s, *e, 1.0)
            } else {
                return Err(LugliError::runtime("range expects numeric arguments"));
            }
        }
        3 => {
            // range(start, stop, step)
            if let (Value::Number(s), Value::Number(e), Value::Number(st)) = (&args[0], &args[1], &args[2]) {
                (*s, *e, *st)
            } else {
                return Err(LugliError::runtime("range expects numeric arguments"));
            }
        }
        _ => {
            return Err(LugliError::runtime("range expects 1, 2, or 3 arguments"));
        }
    };

    if step == 0.0 {
        return Err(LugliError::runtime("range step cannot be zero"));
    }

    let mut result = Vec::new();
    let mut current = start;

    if step > 0.0 {
        while current < stop {
            result.push(Value::Number(current));
            current += step;
        }
    } else {
        while current > stop {
            result.push(Value::Number(current));
            current += step;
        }
    }

    Ok(Value::List(Rc::new(RefCell::new(result))))
}

fn enumerate_fn(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("enumerate expects 1 argument"));
    }

    match &args[0] {
        Value::List(list) => {
            let borrowed = list.borrow();
            let mut result = Vec::new();

            for (i, item) in borrowed.iter().enumerate() {
                let tuple = vec![Value::Number(i as f64), item.clone()];
                result.push(Value::List(Rc::new(RefCell::new(tuple))));
            }

            Ok(Value::List(Rc::new(RefCell::new(result))))
        }
        Value::String(s) => {
            let mut result = Vec::new();

            for (i, ch) in s.chars().enumerate() {
                let tuple = vec![Value::Number(i as f64), Value::String(ch.to_string())];
                result.push(Value::List(Rc::new(RefCell::new(tuple))));
            }

            Ok(Value::List(Rc::new(RefCell::new(result))))
        }
        _ => Err(LugliError::runtime(format!("Cannot enumerate object of type {}", args[0].type_name()))),
    }
}

fn str_fn(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("str expects 1 argument"));
    }
    Ok(Value::String(args[0].to_string()))
}

fn int_fn(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("int expects 1 argument"));
    }
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.floor())),
        Value::String(s) => match s.trim().parse::<f64>() {
            Ok(n) => Ok(Value::Number(n.floor())),
            Err(_) => Err(LugliError::runtime(format!("Cannot convert '{}' to integer", s))),
        },
        Value::Bool(b) => Ok(Value::Number(if *b { 1.0 } else { 0.0 })),
        _ => Err(LugliError::runtime(format!("Cannot convert {} to integer", args[0].type_name()))),
    }
}

fn float_fn(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("float expects 1 argument"));
    }
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(*n)),
        Value::String(s) => match s.trim().parse::<f64>() {
            Ok(n) => Ok(Value::Number(n)),
            Err(_) => Err(LugliError::runtime(format!("Cannot convert '{}' to float", s))),
        },
        Value::Bool(b) => Ok(Value::Number(if *b { 1.0 } else { 0.0 })),
        _ => Err(LugliError::runtime(format!("Cannot convert {} to float", args[0].type_name()))),
    }
}

fn bool_fn(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("bool expects 1 argument"));
    }
    Ok(Value::Bool(args[0].is_truthy()))
}

// Collection utility functions

fn zip_fn(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() < 2 {
        return Err(LugliError::runtime("zip expects at least 2 arguments"));
    }

    // First, validate all args are lists and collect their lengths
    let mut lists: Vec<Vec<Value>> = Vec::new();
    for arg in args {
        match arg {
            Value::List(l) => lists.push(l.borrow().clone()),
            _ => return Err(LugliError::runtime("zip expects all arguments to be lists")),
        }
    }

    let min_len = lists.iter().map(|l| l.len()).min().unwrap_or(0);
    let mut result = Vec::new();

    for i in 0..min_len {
        let tuple: Vec<Value> = lists.iter().map(|l| l[i].clone()).collect();
        result.push(Value::List(Rc::new(RefCell::new(tuple))));
    }

    Ok(Value::List(Rc::new(RefCell::new(result))))
}

fn all_fn(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("all expects 1 argument"));
    }

    match &args[0] {
        Value::List(list) => {
            let all_true = list.borrow().iter().all(|v| v.is_truthy());
            Ok(Value::Bool(all_true))
        }
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

fn any_fn(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("any expects 1 argument"));
    }

    match &args[0] {
        Value::List(list) => {
            let any_true = list.borrow().iter().any(|v| v.is_truthy());
            Ok(Value::Bool(any_true))
        }
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

fn sum_fn(args: &[Value]) -> Result<Value, LugliError> {
    if args.is_empty() || args.len() > 2 {
        return Err(LugliError::runtime("sum expects 1 or 2 arguments"));
    }

    let start = if args.len() == 2 {
        match &args[1] {
            Value::Number(n) => *n,
            _ => return Err(LugliError::runtime("sum start value must be a number")),
        }
    } else {
        0.0
    };

    match &args[0] {
        Value::List(list) => {
            let sum = list.borrow().iter().try_fold(start, |acc, v| match v {
                Value::Number(n) => Ok(acc + n),
                _ => Err(LugliError::runtime("sum expects a list of numbers")),
            })?;
            Ok(Value::Number(sum))
        }
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

fn min_fn(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("min expects 1 argument"));
    }

    match &args[0] {
        Value::List(list) => {
            let borrowed = list.borrow();
            if borrowed.is_empty() {
                return Err(LugliError::runtime("min() arg is an empty sequence"));
            }

            let min = borrowed.iter().try_fold(None, |acc: Option<&Value>, v| match (acc, v) {
                (None, _) => Ok(Some(v)),
                (Some(prev @ Value::Number(a)), Value::Number(b)) => Ok(Some(if a < b { prev } else { v })),
                (Some(prev @ Value::String(a)), Value::String(b)) => Ok(Some(if a < b { prev } else { v })),
                _ => Err(LugliError::runtime("min() expects comparable values")),
            })?;

            match min {
                Some(value) => Ok(value.clone()),
                None => Err(LugliError::runtime("min() internal error: should have been caught by empty check")),
            }
        }
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

fn max_fn(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("max expects 1 argument"));
    }

    match &args[0] {
        Value::List(list) => {
            let borrowed = list.borrow();
            if borrowed.is_empty() {
                return Err(LugliError::runtime("max() arg is an empty sequence"));
            }

            let max = borrowed.iter().try_fold(None, |acc: Option<&Value>, v| match (acc, v) {
                (None, _) => Ok(Some(v)),
                (Some(prev @ Value::Number(a)), Value::Number(b)) => Ok(Some(if a > b { prev } else { v })),
                (Some(prev @ Value::String(a)), Value::String(b)) => Ok(Some(if a > b { prev } else { v })),
                _ => Err(LugliError::runtime("max() expects comparable values")),
            })?;

            match max {
                Some(value) => Ok(value.clone()),
                None => Err(LugliError::runtime("max() internal error: should have been caught by empty check")),
            }
        }
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

// Math functions

fn abs_fn(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("abs expects 1 argument"));
    }
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.abs())),
        _ => Err(LugliError::type_error("number", args[0].type_name())),
    }
}

fn round_fn(args: &[Value]) -> Result<Value, LugliError> {
    if args.is_empty() || args.len() > 2 {
        return Err(LugliError::runtime("round expects 1 or 2 arguments"));
    }

    let num = match &args[0] {
        Value::Number(n) => *n,
        _ => return Err(LugliError::type_error("number", args[0].type_name())),
    };

    if args.len() == 1 {
        // Round to nearest integer
        Ok(Value::Number(num.round()))
    } else {
        // Round to n decimal places
        let places = match &args[1] {
            Value::Number(n) => *n as i32,
            _ => return Err(LugliError::type_error("number", args[1].type_name())),
        };

        let multiplier = 10_f64.powi(places);
        Ok(Value::Number((num * multiplier).round() / multiplier))
    }
}

fn pow_fn(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 2 {
        return Err(LugliError::runtime("pow expects 2 arguments"));
    }

    let base = match &args[0] {
        Value::Number(n) => *n,
        _ => return Err(LugliError::type_error("number", args[0].type_name())),
    };

    let exponent = match &args[1] {
        Value::Number(n) => *n,
        _ => return Err(LugliError::type_error("number", args[1].type_name())),
    };

    Ok(Value::Number(base.powf(exponent)))
}

// Higher-order functions

fn map_fn(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 2 {
        return Err(LugliError::runtime("map expects 2 arguments"));
    }

    let _func = match &args[0] {
        Value::Function { .. } | Value::NativeFunction { .. } | Value::Closure { .. } => &args[0],
        _ => return Err(LugliError::type_error("function", args[0].type_name())),
    };

    match &args[1] {
        Value::List(_list) => {
            // map/filter/reduce require VM integration to call functions
            // For now, return helpful error message
            Err(LugliError::runtime("map is not yet fully implemented - requires VM integration for function calls"))
        }
        _ => Err(LugliError::type_error("list", args[1].type_name())),
    }
}

fn filter_fn(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 2 {
        return Err(LugliError::runtime("filter expects 2 arguments"));
    }

    let _func = match &args[0] {
        Value::Function { .. } | Value::NativeFunction { .. } | Value::Closure { .. } => &args[0],
        _ => return Err(LugliError::type_error("function", args[0].type_name())),
    };

    match &args[1] {
        Value::List(_list) => {
            Err(LugliError::runtime("filter is not yet fully implemented - requires VM integration for function calls"))
        }
        _ => Err(LugliError::type_error("list", args[1].type_name())),
    }
}

fn reduce_fn(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(LugliError::runtime("reduce expects 2 or 3 arguments"));
    }

    let _func = match &args[0] {
        Value::Function { .. } | Value::NativeFunction { .. } | Value::Closure { .. } => &args[0],
        _ => return Err(LugliError::type_error("function", args[0].type_name())),
    };

    match &args[1] {
        Value::List(_list) => {
            Err(LugliError::runtime("reduce is not yet fully implemented - requires VM integration for function calls"))
        }
        _ => Err(LugliError::type_error("list", args[1].type_name())),
    }
}
