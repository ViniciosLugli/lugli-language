use crate::validation::*;
use crate::NativeFunction;
use lugli_common::{LugliError, StringPool, Value};
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
        ("ceil", number::number_ceil as NativeFunction),
        ("floor", number::number_floor as NativeFunction),
        ("sqrt", number::number_sqrt as NativeFunction),
        ("sin", number::number_sin as NativeFunction),
        ("cos", number::number_cos as NativeFunction),
        ("tan", number::number_tan as NativeFunction),
        ("log", number::number_log as NativeFunction),
        ("exp", number::number_exp as NativeFunction),
        // Higher-order functions
        ("map", map_fn as NativeFunction),
        ("filter", filter_fn as NativeFunction),
        ("reduce", reduce_fn as NativeFunction),
    ];

    // Add string methods
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

fn type_of(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "type")?;
    let type_name = args[0].type_name();
    Ok(Value::String(pool.intern(type_name)))
}

fn len_fn(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "len")?;
    match &args[0] {
        Value::String(id) => Ok(Value::Number(pool.resolve(*id).len() as f64)),
        Value::List(l) => Ok(Value::Number(l.borrow().len() as f64)),
        Value::Dict(d) => Ok(Value::Number(d.borrow().len() as f64)),
        _ => Err(LugliError::runtime(format!("Object of type {} has no len()", args[0].type_name()))),
    }
}

fn range_fn(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    let (start, stop, step) = match args.len() {
        1 => {
            if let Value::Number(n) = args[0] {
                (0.0, n, 1.0)
            } else {
                return Err(LugliError::runtime("range expects numeric arguments"));
            }
        }
        2 => {
            if let (Value::Number(s), Value::Number(e)) = (&args[0], &args[1]) {
                (*s, *e, 1.0)
            } else {
                return Err(LugliError::runtime("range expects numeric arguments"));
            }
        }
        3 => {
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

fn enumerate_fn(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "enumerate")?;
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
        Value::String(id) => {
            let s = pool.resolve(*id).to_string();
            let mut result = Vec::new();

            for (i, ch) in s.chars().enumerate() {
                let ch_id = pool.intern(&ch.to_string());
                let tuple = vec![Value::Number(i as f64), Value::String(ch_id)];
                result.push(Value::List(Rc::new(RefCell::new(tuple))));
            }

            Ok(Value::List(Rc::new(RefCell::new(result))))
        }
        _ => Err(LugliError::runtime(format!("Cannot enumerate object of type {}", args[0].type_name()))),
    }
}

fn str_fn(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "str")?;
    let s = args[0].display_with_pool(pool);
    Ok(Value::String(pool.intern(&s)))
}

fn int_fn(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "int")?;
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.floor())),
        Value::String(id) => {
            let s = pool.resolve(*id);
            match s.trim().parse::<f64>() {
                Ok(n) => Ok(Value::Number(n.floor())),
                Err(_) => Err(LugliError::runtime(format!("Cannot convert '{}' to integer", s))),
            }
        }
        Value::Bool(b) => Ok(Value::Number(if *b { 1.0 } else { 0.0 })),
        _ => Err(LugliError::runtime(format!("Cannot convert {} to integer", args[0].type_name()))),
    }
}

fn float_fn(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "float")?;
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(*n)),
        Value::String(id) => {
            let s = pool.resolve(*id);
            match s.trim().parse::<f64>() {
                Ok(n) => Ok(Value::Number(n)),
                Err(_) => Err(LugliError::runtime(format!("Cannot convert '{}' to float", s))),
            }
        }
        Value::Bool(b) => Ok(Value::Number(if *b { 1.0 } else { 0.0 })),
        _ => Err(LugliError::runtime(format!("Cannot convert {} to float", args[0].type_name()))),
    }
}

fn bool_fn(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "bool")?;
    Ok(Value::Bool(args[0].is_truthy()))
}

fn zip_fn(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_min_arity(args, 2, "zip")?;
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

fn all_fn(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
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

fn any_fn(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "any")?;
    match &args[0] {
        Value::List(list) => {
            let any_true = list.borrow().iter().any(|v| v.is_truthy());
            Ok(Value::Bool(any_true))
        }
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

fn sum_fn(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity_range(args, 1, 2, "sum")?;
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

enum Comparison {
    Min,
    Max,
}

fn minmax_fn(args: &[Value], pool: &mut StringPool, op: Comparison) -> Result<Value, LugliError> {
    let fn_name = match op {
        Comparison::Min => "min",
        Comparison::Max => "max",
    };

    if args.len() != 1 {
        return Err(LugliError::runtime(format!("{} expects 1 argument", fn_name)));
    }

    match &args[0] {
        Value::List(list) => {
            let borrowed = list.borrow();
            if borrowed.is_empty() {
                return Err(LugliError::runtime(format!("{}() arg is an empty sequence", fn_name)));
            }

            let result = borrowed.iter().try_fold(None, |acc: Option<&Value>, v| match (acc, v) {
                (None, _) => Ok(Some(v)),
                (Some(prev @ Value::Number(a)), Value::Number(b)) => {
                    let choose_prev = match op {
                        Comparison::Min => a < b,
                        Comparison::Max => a > b,
                    };
                    Ok(Some(if choose_prev { prev } else { v }))
                }
                (Some(prev @ Value::String(a)), Value::String(b)) => {
                    let a_str = pool.resolve(*a);
                    let b_str = pool.resolve(*b);
                    let choose_prev = match op {
                        Comparison::Min => a_str < b_str,
                        Comparison::Max => a_str > b_str,
                    };
                    Ok(Some(if choose_prev { prev } else { v }))
                }
                _ => Err(LugliError::runtime(format!("{}() expects comparable values", fn_name))),
            })?;

            match result {
                Some(value) => Ok(value.clone()),
                None => Err(LugliError::runtime(format!("{}() internal error: should have been caught by empty check", fn_name))),
            }
        }
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}

fn min_fn(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    minmax_fn(args, pool, Comparison::Min)
}

fn max_fn(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    minmax_fn(args, pool, Comparison::Max)
}

fn abs_fn(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "abs")?;
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.abs())),
        _ => Err(LugliError::type_error("number", args[0].type_name())),
    }
}

fn round_fn(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity_range(args, 1, 2, "round")?;

    let num = match &args[0] {
        Value::Number(n) => *n,
        _ => return Err(LugliError::type_error("number", args[0].type_name())),
    };

    if args.len() == 1 {
        Ok(Value::Number(num.round()))
    } else {
        let places = match &args[1] {
            Value::Number(n) => *n as i32,
            _ => return Err(LugliError::type_error("number", args[1].type_name())),
        };

        let multiplier = 10_f64.powi(places);
        Ok(Value::Number((num * multiplier).round() / multiplier))
    }
}

fn pow_fn(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
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

fn map_fn(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 2, "map")?;
    let _func = match &args[0] {
        Value::Function {
            ..
        }
        | Value::NativeFunction {
            ..
        }
        | Value::Closure {
            ..
        } => &args[0],
        _ => return Err(LugliError::type_error("function", args[0].type_name())),
    };

    match &args[1] {
        Value::List(_list) => Err(LugliError::runtime("map is not yet fully implemented - requires VM integration for function calls")),
        _ => Err(LugliError::type_error("list", args[1].type_name())),
    }
}

fn filter_fn(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 2, "filter")?;
    let _func = match &args[0] {
        Value::Function {
            ..
        }
        | Value::NativeFunction {
            ..
        }
        | Value::Closure {
            ..
        } => &args[0],
        _ => return Err(LugliError::type_error("function", args[0].type_name())),
    };

    match &args[1] {
        Value::List(_list) => Err(LugliError::runtime("filter is not yet fully implemented - requires VM integration for function calls")),
        _ => Err(LugliError::type_error("list", args[1].type_name())),
    }
}

fn reduce_fn(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity_range(args, 2, 3, "reduce")?;
    let _func = match &args[0] {
        Value::Function {
            ..
        }
        | Value::NativeFunction {
            ..
        }
        | Value::Closure {
            ..
        } => &args[0],
        _ => return Err(LugliError::type_error("function", args[0].type_name())),
    };

    match &args[1] {
        Value::List(_list) => Err(LugliError::runtime("reduce is not yet fully implemented - requires VM integration for function calls")),
        _ => Err(LugliError::type_error("list", args[1].type_name())),
    }
}
