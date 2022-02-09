use std::io::Write;
use rust_decimal::prelude::ToPrimitive;
use super::value::Value;
use crate::error::{Result, Location};

pub fn print<T: Write>(args: Vec<Value>, stdout: &mut T, start_loc: Location, end_loc: Location) -> Result<Value> {
    let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
    match write!(stdout, "{}", args.join(" ")) {
        Ok(_) => {},
        Err(e) => {
            error!(SystemError, start_loc, end_loc, "Failed while writing to stdout: {}", e);
        },
    };
    return Ok(Value::Null);
}

pub fn printl<T: Write>(args: Vec<Value>, stdout: &mut T, start_loc: Location, end_loc: Location) -> Result<Value> {
    let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
    match writeln!(stdout, "{}", args.join(" ")) {
        Ok(_) => {},
        Err(e) => {
            error!(SystemError, start_loc, end_loc, "Failed while writing to stdout: {}", e);
        },
    };
    return Ok(Value::Null);
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

pub fn exit(args: Vec<Value>, callback: fn(i32), start_loc: Location, end_loc: Location) -> Result<Value> {
    if args.len() != 1 {
        error!(
            TypeError,
            start_loc,
            end_loc,
            "Function 'exit' takes 1 argument, however {} were supplied",
            args.len(),
        );
    }
    match args[0] {
        Value::Number(num) => {
            if !num.fract().is_zero() {
                error!(
                    ValueError,
                    start_loc,
                    end_loc,
                    "Exit code has to be an integer",
                )
            }
            match num.to_i32() {
                Some(num) => callback(num),
                _ => error!(
                    ValueError,
                    start_loc,
                    end_loc,
                    "Exit code is too high or too low",
                )
            }
        },
        _ => error!(
            TypeError,
            start_loc,
            end_loc,
            "First argument of function 'exit' has to be of type number",
        ),
    }
    return Ok(Value::Null);
}
