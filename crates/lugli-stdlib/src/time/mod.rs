use crate::NativeFunction;
use chrono::Utc;
use lugli_common::{LugliError, StringPool, Value};

pub mod datetime;

pub fn get_functions() -> Vec<(&'static str, NativeFunction)> { vec![("now", time_now), ("sleep", time_sleep)] }

fn time_now(_args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    let now = Utc::now();
    Ok(Value::Number(now.timestamp() as f64))
}

fn time_sleep(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("sleep expects 1 argument"));
    }
    match &args[0] {
        Value::Number(ms) => {
            std::thread::sleep(std::time::Duration::from_millis(*ms as u64));
            Ok(Value::Null)
        }
        _ => Err(LugliError::type_error("number", args[0].type_name())),
    }
}
