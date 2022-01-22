pub mod types;
pub mod calculative_operations;
pub mod relational_operations;
pub mod truth;

use std::fmt::Display;
use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use crate::nodes::Statements;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value {
    Int(BigInt),
    Float(BigDecimal),
    Bool(bool),
    String(String),
    Range(bool, BigInt, BigInt),
    Function(Vec<String>, Statements),
    BuiltIn,
    Null,
    Void,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Value::Int(value)    => value.to_string(),
            Value::Float(value)  => value.to_string(),
            Value::Bool(value)   => value.to_string(),
            Value::String(value) => value.to_string(),
            Value::Range(inclusive, start, end) => format!(
                "{}..{}{}",
                start,
                if *inclusive { "=" } else { "" },
                end,
            ),
            Value::Function(_, _) => String::from("<function>"),
            Value::BuiltIn        => String::from("<built-in>"),
            Value::Null           => String::from("null"),
            Value::Void           => String::from("void"),
        })
    }
}
