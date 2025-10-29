use chrono::{TimeZone, Utc};
use lugli_common::{LugliError, StringPool, Value};

pub fn format_datetime(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 2 {
        return Err(LugliError::runtime("format_datetime expects 2 arguments"));
    }

    let timestamp = match &args[0] {
        Value::Number(n) => *n as i64,
        _ => return Err(LugliError::type_error("number", args[0].type_name())),
    };

    let format = match &args[1] {
        Value::String(id) => pool.resolve(*id),
        _ => return Err(LugliError::type_error("string", args[1].type_name())),
    };

    let dt = Utc.timestamp_opt(timestamp, 0).single().ok_or_else(|| LugliError::runtime("Invalid timestamp"))?;

    let formatted = dt.format(format).to_string();
    Ok(Value::String(pool.intern(&formatted)))
}
