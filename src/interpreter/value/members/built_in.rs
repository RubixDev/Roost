use std::str::FromStr;
use rust_decimal::{Decimal, prelude::ToPrimitive};
use crate::{error::{Result, Location}, interpreter::value::{Value, truth::Truth}};

macro_rules! want_num_args {
    ($args:ident, $num:expr, $name:expr, $start:ident, $end:ident) => {
        if $args.len() != $num {
            error!(
                TypeError,
                $start,
                $end,
                "Function '{}' takes 1 argument, however {} were supplied",
                $name,
                $args.len(),
            );
        }
    };
}

macro_rules! cannot_to {
    ($self:ident, $start:ident, $end:ident, $to:expr) => {
        error!(ValueError, $start, $end, "Could not parse String '{}' to {}", $self, $to)
    };
}

pub fn to_string(self_: Value, args: Vec<Value>, start: Location, end: Location) -> Result<Value> {
    want_num_args!(args, 0, "toString", start, end);
    return Ok(Value::String(self_.to_string()));
}

pub fn to_bool(self_: Value, args: Vec<Value>, start: Location, end: Location) -> Result<Value> {
    want_num_args!(args, 0, "toBool", start, end);
    return Ok(Value::Bool(self_.is_true()));
}

pub fn str_to_int(self_: Value, args: Vec<Value>, start: Location, end: Location) -> Result<Value> {
    if args.len() > 1 { want_num_args!(args, 1, "toInt", start, end); }
    let radix = match args.get(0) {
        Some(radix) => match radix {
            Value::Number(num) => match num.to_u32() {
                Some(num) => num,
                None => error!(ValueError, start, end, "Invalid radix: {}", radix),
            },
            _ => error!(TypeError, start, end, "The radix has to be of type number"),
        },
        None => 10,
    };
    if !(2..=36).contains(&radix) { error!(ValueError, start, end, "Radix has to be in 2..=36, got {}", radix); }
    if self_.to_string().contains('.') { cannot_to!(self_, start, end, "integer"); }
    return match Decimal::from_str_radix(&self_.to_string(), radix) {
        Ok(int) => Ok(Value::Number(int)),
        Err(_) => cannot_to!(self_, start, end, "integer"),
    };
}

pub fn str_to_decimal(self_: Value, args: Vec<Value>, start: Location, end: Location) -> Result<Value> {
    want_num_args!(args, 0, "toDecimal", start, end);
    return match Decimal::from_str(&self_.to_string()) {
        Ok(num) => Ok(Value::Number(num)),
        Err(_) => cannot_to!(self_, start, end, "decimal number"),
    };
}

pub fn str_to_bool(self_: Value, args: Vec<Value>, start: Location, end: Location) -> Result<Value> {
    want_num_args!(args, 0, "toBool", start, end);
    return Ok(Value::Bool(self_.to_string().to_ascii_lowercase() == "true"));
}

pub fn str_to_bool_strict(self_: Value, args: Vec<Value>, start: Location, end: Location) -> Result<Value> {
    want_num_args!(args, 0, "toBoolStrict", start, end);
    return Ok(match self_.to_string().as_str() {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => cannot_to!(self_, start, end, "bool"),
    });
}

pub fn str_to_range(self_: Value, args: Vec<Value>, start: Location, end: Location) -> Result<Value> {
    want_num_args!(args, 0, "toRange", start, end);
    let self_str = self_.to_string();
    if !self_str.contains("..") { cannot_to!(self_, start, end, "range"); }
    let mut parts = self_str.split("..");
    let num1 = parts.next().unwrap();
    let mut num2 = parts.next().unwrap();
    if parts.next() != None { cannot_to!(self_, start, end, "range"); }

    if num1.contains('.') { cannot_to!(self_, start, end, "range"); }
    let num1 = match Decimal::from_str(num1) {
        Ok(num) => num,
        Err(_) => cannot_to!(self_, start, end, "range"),
    };
    let inclusive = if num2.starts_with('=') {
        num2 = &num2[1..];
        true
    } else { false };
    if num2.contains('.') { cannot_to!(self_, start, end, "range"); }
    let num2 = match Decimal::from_str(num2) {
        Ok(num) => num,
        Err(_) => cannot_to!(self_, start, end, "range"),
    };
    return Ok(if !inclusive && num1 != num2 {
        if num1 > num2 {
            Value::Range(num1, num2 + Decimal::ONE)
        } else {
            Value::Range(num1, num2 - Decimal::ONE)
        }
    } else { Value::Range(num1, num2) });
}

pub fn str_to_uppercase(self_: Value, args: Vec<Value>, start: Location, end: Location) -> Result<Value> {
    want_num_args!(args, 0, "toUppercase", start, end);
    return Ok(Value::String(self_.to_string().to_ascii_uppercase()));
}

pub fn str_to_lowercase(self_: Value, args: Vec<Value>, start: Location, end: Location) -> Result<Value> {
    want_num_args!(args, 0, "toLowercase", start, end);
    return Ok(Value::String(self_.to_string().to_ascii_lowercase()));
}

pub fn num_to_int(self_: Value, args: Vec<Value>, start: Location, end: Location) -> Result<Value> {
    want_num_args!(args, 0, "toInt", start, end);
    let num = if let Value::Number(num) = self_ { num } else { panic!() };
    return Ok(Value::Number(num.trunc()));
}

pub fn num_floor(self_: Value, args: Vec<Value>, start: Location, end: Location) -> Result<Value> {
    want_num_args!(args, 0, "toInt", start, end);
    let num = if let Value::Number(num) = self_ { num } else { panic!() };
    return Ok(Value::Number(num.floor()));
}

pub fn num_ceil(self_: Value, args: Vec<Value>, start: Location, end: Location) -> Result<Value> {
    want_num_args!(args, 0, "toInt", start, end);
    let num = if let Value::Number(num) = self_ { num } else { panic!() };
    return Ok(Value::Number(num.ceil()));
}

pub fn num_round(self_: Value, args: Vec<Value>, start: Location, end: Location) -> Result<Value> {
    want_num_args!(args, 0, "toInt", start, end);
    let num = if let Value::Number(num) = self_ { num } else { panic!() };
    return Ok(Value::Number(num.round()));
}
