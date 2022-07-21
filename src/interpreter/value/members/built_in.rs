use std::str::FromStr;

use crate::{
    error::{Result, Span},
    interpreter::value::{Value, WrappedValue},
};
use rust_decimal::{prelude::ToPrimitive, Decimal};

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

macro_rules! parse_err {
    ($this:ident, $span:ident, $to:expr) => {
        error!(
            ValueError,
            $span,
            "Could not parse String '{}' to {}",
            $this.borrow(),
            $to,
        )
    };
}

macro_rules! unwrap_variant {
    ($borrow:expr, $variant:ident) => {
        match &*$borrow {
            Value::$variant(val) => val,
            _ => unreachable!(),
        }
    };
}

pub fn to_string<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: Span,
) -> Result<Value<'tree>> {
    expect_len!(args, 0, "toString", span);
    Ok(Value::String(this.borrow().to_string()))
}

pub fn to_bool<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: Span,
) -> Result<Value<'tree>> {
    expect_len!(args, 0, "toBool", span);
    Ok(Value::Bool(this.borrow().is_true()))
}

pub fn clone<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: Span,
) -> Result<Value<'tree>> {
    expect_len!(args, 0, "clone", span);
    Ok(this.borrow().clone())
}

pub fn str_to_int<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: Span,
) -> Result<Value<'tree>> {
    let borrow = this.borrow();
    let str = unwrap_variant!(borrow, String);
    if args.len() > 1 {
        expect_len!(args, 1, "toInt", span);
    }
    let radix = match args.get(0) {
        Some(val) => match &*val.borrow() {
            Value::Number(radix) => match radix.to_u32() {
                Some(num) => num,
                None => error!(ValueError, span, "Invalid radix: {}", radix),
            },
            _ => error!(TypeError, span, "The radix has to be of type 'number'",),
        },
        None => 10,
    };
    if !(2..=36).contains(&radix) {
        error!(ValueError, span, "Radix has to be in 2..=36, got {}", radix,);
    }
    match Decimal::from_str_radix(str, radix) {
        Ok(num) if num.fract().is_zero() => Ok(Value::Number(num)),
        _ => parse_err!(this, span, "integer"),
    }
}

pub fn str_to_number<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: Span,
) -> Result<Value<'tree>> {
    let borrow = this.borrow();
    let str = unwrap_variant!(borrow, String);
    expect_len!(args, 0, "toNumber", span);
    match Decimal::from_str(str) {
        Ok(num) => Ok(Value::Number(num)),
        Err(_) => parse_err!(this, span, "number"),
    }
}

pub fn str_to_bool<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: Span,
) -> Result<Value<'tree>> {
    let borrow = this.borrow();
    let str = unwrap_variant!(borrow, String);
    expect_len!(args, 0, "toBool", span);
    Ok(Value::Bool(str.to_ascii_lowercase() == "true"))
}

pub fn str_to_bool_strict<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: Span,
) -> Result<Value<'tree>> {
    let borrow = this.borrow();
    let str = unwrap_variant!(borrow, String);
    expect_len!(args, 0, "toBoolStrict", span);
    Ok(match str.as_str() {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => parse_err!(this, span, "bool"),
    })
}

pub fn str_to_range<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: Span,
) -> Result<Value<'tree>> {
    let borrow = this.borrow();
    let str = unwrap_variant!(borrow, String);
    expect_len!(args, 0, "toRange", span);
    let mut split = str.split("..");
    let left = split.next().unwrap();
    let mut right = match split.next() {
        Some(right) => right,
        None => parse_err!(this, span, "range"),
    };
    let inclusive = right.starts_with('=');
    if inclusive {
        right = &right[1..];
    }

    let (left, right) = match (i128::from_str(left), i128::from_str(right)) {
        (Ok(left), Ok(right)) => (left, right),
        _ => parse_err!(this, span, "range"),
    };
    Ok(Value::Range {
        start: left,
        end: right - !inclusive as i128,
    })
}

pub fn str_to_uppercase<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: Span,
) -> Result<Value<'tree>> {
    let borrow = this.borrow();
    let str = unwrap_variant!(borrow, String);
    expect_len!(args, 0, "toUppercase", span);
    Ok(Value::String(str.to_ascii_uppercase()))
}

pub fn str_to_lowercase<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: Span,
) -> Result<Value<'tree>> {
    let borrow = this.borrow();
    let str = unwrap_variant!(borrow, String);
    expect_len!(args, 0, "toLowercase", span);
    Ok(Value::String(str.to_ascii_lowercase()))
}

pub fn num_to_int<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: Span,
) -> Result<Value<'tree>> {
    let borrow = this.borrow();
    let num = unwrap_variant!(borrow, Number);
    expect_len!(args, 0, "toInt", span);
    Ok(Value::Number(num.trunc()))
}

pub fn num_floor<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: Span,
) -> Result<Value<'tree>> {
    let borrow = this.borrow();
    let num = unwrap_variant!(borrow, Number);
    expect_len!(args, 0, "floor", span);
    Ok(Value::Number(num.floor()))
}

pub fn num_ceil<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: Span,
) -> Result<Value<'tree>> {
    let borrow = this.borrow();
    let num = unwrap_variant!(borrow, Number);
    expect_len!(args, 0, "ceil", span);
    Ok(Value::Number(num.ceil()))
}

pub fn num_round<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: Span,
) -> Result<Value<'tree>> {
    let borrow = this.borrow();
    let num = unwrap_variant!(borrow, Number);
    expect_len!(args, 0, "round", span);
    Ok(Value::Number(num.round()))
}
