use super::value::Value;
use crate::error::{Result, Location};

pub fn print(args: Vec<Value>, callback: fn(String)) -> Result<Value> {
    let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
    callback(args.join(" "));
    return Ok(Value::Void);
}

pub fn printl(args: Vec<Value>, callback: fn(String)) -> Result<Value> {
    let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
    callback(args.join(" ") + "\n");
    return Ok(Value::Void);
}

pub fn type_of(args: Vec<Value>, start_loc: Location, end_loc: Location) -> Result<Value> {
    if args.len() != 1 {
        error!(
            TypeError,
            start_loc,
            end_loc,
            "Function 'typeOf' takes 1 argument, however {} were supplied",
            args.len(),
        );
    }

    return Ok(Value::String(super::value::types::type_of(&args[0]).to_string()));
}
