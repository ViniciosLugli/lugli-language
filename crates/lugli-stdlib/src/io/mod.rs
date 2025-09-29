use lugli_common::{Value, LugliError};
use crate::NativeFunction;

pub mod console;

pub fn get_functions() -> Vec<(&'static str, NativeFunction)> {
    vec![
        ("print", print_fn),
        ("println", println_fn),
    ]
}

fn print_fn(args: &[Value]) -> Result<Value, LugliError> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    Ok(Value::Null)
}

fn println_fn(args: &[Value]) -> Result<Value, LugliError> {
    print_fn(args)?;
    println!();
    Ok(Value::Null)
}