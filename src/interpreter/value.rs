pub mod types;
pub mod calculative_operations;
pub mod relational_operations;
pub mod truth;
pub mod iterator;
pub mod members;

use std::fmt::{Display, Debug};
use rust_decimal::Decimal;
use crate::{nodes::Statements, error::{Result, Location}};

#[derive(PartialEq, Clone)]
pub enum Value {
    Number(Decimal),
    Bool(bool),
    String(String),
    Range(Decimal, Decimal),
    Function(Vec<String>, Statements),
    BuiltIn(BuiltIn),
    Null,
}

#[derive(PartialEq, Clone)]
pub enum BuiltIn {
    Special(SpecialBuiltIn),
    Function(fn (args: Vec<Value>, start_loc: Location, end_loc: Location) -> Result<Value>),
    Method(fn (self_: Value, args: Vec<Value>, start_loc: Location, end_loc: Location) -> Result<Value>),
}
#[derive(PartialEq, Clone)]
pub enum SpecialBuiltIn {
    Print(bool),
    Exit,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Value::Number(value)     => value.to_string(),
            Value::Bool(value)       => value.to_string(),
            Value::String(value)     => value.to_string(),
            Value::Range(start, end) => format!("{}..={}", start, end),
            Value::Function(_, _)    => String::from("<function>"),
            Value::BuiltIn(_)        => String::from("<built-in>"),
            Value::Null              => String::from("null"),
        })
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Value::Number(value)     => format!("\x1b[33m{}\x1b[0m", value),
            Value::Bool(value)       => format!("\x1b[34m{}\x1b[0m", value),
            Value::String(value)     => format!("\x1b[32m'{}'\x1b[0m", value),
            Value::Range(start, end) => format!("\x1b[33m{}\x1b[0m..=\x1b[33m{}\x1b[0m", start, end),
            Value::Function(_, _)    => String::from("\x1b[90m<function>\x1b[0m"),
            Value::BuiltIn(_)        => String::from("\x1b[90m<built-in>\x1b[0m"),
            Value::Null              => String::from("\x1b[90mnull\x1b[0m"),
        })
    }
}
