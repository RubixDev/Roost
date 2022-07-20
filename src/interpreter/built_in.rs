use rust_decimal::prelude::ToPrimitive;

use crate::error::{Location, Result};
#[cfg(feature = "no_std_io")]
use crate::io::Write;
#[cfg(not(feature = "no_std_io"))]
use std::io::Write;

use super::value::{types, Value, WrappedValue};

#[cfg(not(feature = "no_std_io"))]
pub fn print<'tree>(
    args: Vec<WrappedValue<'tree>>,
    stdout: &mut impl Write,
    start: &Location,
    end: &Location,
    newline: bool,
) -> Result<Value<'tree>> {
    let args: Vec<String> = args.iter().map(|arg| arg.borrow().to_string()).collect();
    if let Err(e) = write!(
        stdout,
        "{}{}",
        args.join(" "),
        if newline { "\n" } else { "" }
    ) {
        error!(
            SystemError,
            *start, *end, "Failed to write to stdout: {}", e
        );
    }
    Ok(Value::Null)
}

#[cfg(feature = "no_std_io")]
pub fn print<'tree>(
    args: Vec<WrappedValue<'tree>>,
    stdout: &mut impl Write,
    start: &Location,
    end: &Location,
    newline: bool,
) -> Result<Value<'tree>> {
    let args: Vec<String> = args.iter().map(|arg| arg.borrow().to_string()).collect();
    stdout.write(format!(
        "{}{}",
        args.join(" "),
        if newline { "\n" } else { "" }
    ));
    Ok(Value::Null)
}

pub fn exit<'tree>(
    args: Vec<WrappedValue<'tree>>,
    callback: impl FnOnce(i32),
    start: &Location,
    end: &Location,
) -> Result<Value<'tree>> {
    if args.len() != 1 {
        error!(
            TypeError,
            *start,
            *end,
            "Function 'exit' takes 1 argument, however {} were supplied",
            args.len(),
        );
    }
    if let Value::Number(num) = &*args[0].borrow() {
        if !num.fract().is_zero() {
            error!(ValueError, *start, *end, "Exit code has to be an integer");
        }
        if let Some(num) = num.to_i32() {
            callback(num)
        } else {
            error!(ValueError, *start, *end, "Exit code is too high or too low");
        }
    } else {
        error!(
            TypeError,
            *start, *end, "First argument of function 'exit' has to be of type 'number'",
        );
    }
    Ok(Value::Null)
}

pub fn type_of<'tree>(
    args: Vec<WrappedValue<'tree>>,
    start: &Location,
    end: &Location,
) -> Result<Value<'tree>> {
    if args.len() != 1 {
        error!(
            TypeError,
            *start,
            *end,
            "Function 'typeOf' takes 1 argument, however {} were supplied",
            args.len(),
        );
    }
    Ok(Value::String(types::type_of(&args[0].borrow()).to_string()))
}
