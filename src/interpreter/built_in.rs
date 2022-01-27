use super::value::Value;
use crate::error::{Result, Location};

pub fn print(args: Vec<Value>) -> Result<Value> {
    let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
    print!("{}", args.join(" "));
    return Ok(Value::Void);
}

pub fn printl(args: Vec<Value>) -> Result<Value> {
    let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
    println!("{}", args.join(" "));
    return Ok(Value::Void);
}

pub fn type_of(args: Vec<Value>, location: Location) -> Result<Value> {
    if args.len() != 1 {
        error!(
            TypeError,
            location,
            "Function 'typeOf' takes 1 argument, however {} were supplied",
            args.len(),
        );
    }

    return Ok(Value::String(super::value::types::type_of(&args[0]).to_string()));
}
