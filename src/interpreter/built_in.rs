use rust_decimal::prelude::ToPrimitive;

use crate::error::{Result, Span};
#[cfg(feature = "no_std_io")]
use crate::io::Write;
#[cfg(not(feature = "no_std_io"))]
use std::io::Write;

use super::value::{types, Value, WrappedValue};

#[macro_export]
macro_rules! expect_len {
    ($args:ident, $num:literal, $name:literal, $span:ident) => {
        if $args.len() != $num {
            error!(
                TypeError,
                $span,
                concat!(
                    "Function '", $name, "' takes ", $num, " argument", expect_len!(@plural $num),
                    ", however {} were supplied"
                ),
                $args.len(),
            );
        }
    };
    (@plural 1) => { "" };
    (@plural $num:literal) => { "s" };
}

#[cfg(not(feature = "no_std_io"))]
pub fn print<'tree>(
    args: Vec<WrappedValue<'tree>>,
    stdout: &mut impl Write,
    span: Span,
    newline: bool,
) -> Result<Value<'tree>> {
    let args: Vec<String> = args.iter().map(|arg| arg.borrow().to_string()).collect();
    if let Err(e) = write!(
        stdout,
        "{}{}",
        args.join(" "),
        if newline { "\n" } else { "" }
    ) {
        error!(SystemError, span, "Failed to write to stdout: {}", e,);
    }
    Ok(Value::Null)
}

#[cfg(feature = "no_std_io")]
pub fn print<'tree>(
    args: Vec<WrappedValue<'tree>>,
    stdout: &mut impl Write,
    _span: Span,
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

pub fn exit(args: Vec<WrappedValue>, callback: impl FnOnce(i32), span: Span) -> Result<Value> {
    expect_len!(args, 0, "exit", span);
    if let Value::Number(num) = &*args[0].borrow() {
        if !num.fract().is_zero() {
            error!(ValueError, span, "Exit code has to be an integer");
        }
        if let Some(num) = num.to_i32() {
            callback(num)
        } else {
            error!(ValueError, span, "Exit code is too high or too low");
        }
    } else {
        error!(
            TypeError,
            span, "First argument of function 'exit' has to be of type 'number'",
        );
    }
    Ok(Value::Null)
}

pub fn type_of(args: Vec<WrappedValue>, span: Span) -> Result<Value> {
    expect_len!(args, 1, "typeOf", span);
    Ok(Value::String(types::type_of(&args[0].borrow()).to_string()))
}

pub fn assert(args: Vec<WrappedValue>, span: Span) -> Result<Value> {
    expect_len!(args, 1, "assert", span);
    if args[0].borrow().is_false() {
        error!(RuntimeError, span, "Assertion failed",);
    }
    Ok(Value::Null)
}

pub fn throw(args: Vec<WrappedValue>, span: Span) -> Result<Value> {
    expect_len!(args, 1, "throw", span);
    let borrow = args[0].borrow();
    let str = match &*borrow {
        Value::String(str) => str,
        _ => error!(
            TypeError,
            span, "First argument of function 'throw' has to be of type 'string'",
        ),
    };
    error!(RuntimeError, span, "{str}",)
}
