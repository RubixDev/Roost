use std::fmt::Display;
use super::Value;

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Int,
    Long,
    Float,
    Decimal,
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
            Type::Int      => "int",
            Type::Long     => "long",
            Type::Float    => "float",
            Type::Decimal  => "decimal",
            Type::Bool     => "bool",
            Type::String   => "string",
            Type::Range    => "range",
            Type::Function => "function",
            Type::Null     => "null",
            Type::Void     => "void",
        })
    }
}

pub fn type_of(value: &Value) -> Type {
    return match value {
        Value::Int(_)         => Type::Int,
        Value::Long(_)        => Type::Long,
        Value::Float(_)       => Type::Float,
        Value::Decimal(_)     => Type::Decimal,
        Value::Bool(_)        => Type::Bool,
        Value::String(_)      => Type::String,
        Value::Range(_)       => Type::Range,
        Value::Function(_, _)
        | Value::BuiltIn      => Type::Function,
        Value::Null           => Type::Null,
        Value::Void           => Type::Void,
    };
}
