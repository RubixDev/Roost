pub mod types;
pub mod calculative_operations;
pub mod relational_operations;
pub mod truth;

use std::fmt::Display;
use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use crate::nodes::Statements;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Int(i64),
    Long(BigInt),
    Float(f64),
    Decimal(BigDecimal),
    Bool(bool),
    String(String),
    Range(Range),
    Function(Vec<String>, Statements),
    BuiltIn,
    Null,
    Void,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Range {
    Int(bool, i64, i64),
    Long(bool, BigInt, BigInt),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Value::Int(value)     => value.to_string(),
            Value::Long(value)    => value.to_string(),
            Value::Float(value)   => value.to_string(),
            Value::Decimal(value) => value.to_string(),
            Value::Bool(value)    => value.to_string(),
            Value::String(value)  => value.to_string(),
            Value::Range(value) => format!(
                "{}..{}{}",
                match value {
                    Range::Int(_, start, _) => start.to_string(),
                    Range::Long(_, start, _) => start.to_string(),
                },
                if match value {
                    Range::Int(inclusive, _, _) => *inclusive,
                    Range::Long(inclusive, _, _) => *inclusive,
                } { "=" } else { "" },
                match value {
                    Range::Int(_, _, end) => end.to_string(),
                    Range::Long(_, _, end) => end.to_string(),
                },
            ),
            Value::Function(_, _) => String::from("<function>"),
            Value::BuiltIn        => String::from("<built-in>"),
            Value::Null           => String::from("null"),
            Value::Void           => String::from("void"),
        })
    }
}
