pub mod types;
pub mod calculative_operations;
pub mod relational_operations;
pub mod truth;
pub mod iterator;

use std::fmt::Display;
use rust_decimal::Decimal;

use crate::nodes::Statements;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Number(Decimal),
    Bool(bool),
    String(String),
    Range(Decimal, Decimal),
    Function(Vec<String>, Statements),
    BuiltIn,
    Null,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Value::Number(value)     => value.to_string(),
            Value::Bool(value)       => value.to_string(),
            Value::String(value)     => value.to_string(),
            Value::Range(start, end) => format!("{}..={}", start, end),
            Value::Function(_, _)    => String::from("<function>"),
            Value::BuiltIn           => String::from("<built-in>"),
            Value::Null              => String::from("null"),
        })
    }
}
