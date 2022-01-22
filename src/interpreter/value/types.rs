use std::fmt::Display;
use super::Value;

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Range,
    Function,
    Null,
    Void,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Type::Int       => "int",
            Type::Float     => "float",
            Type::Bool      => "bool",
            Type::String    => "string",
            Type::Range     => "range",
            Type::Function  => "function",
            Type::Null      => "null",
            Type::Void      => "void",
        })
    }
}

pub fn type_of(value: &Value) -> Type {
    return match value {
        Value::Int(_)         => Type::Int,
        Value::Float(_)       => Type::Float,
        Value::Bool(_)        => Type::Bool,
        Value::String(_)      => Type::String,
        Value::Range(_, _, _) => Type::Range,
        Value::Function(_, _)
        | Value::BuiltIn      => Type::Function,
        Value::Null           => Type::Null,
        Value::Void           => Type::Void,
    };
}
