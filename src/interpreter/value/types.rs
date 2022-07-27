use super::Value;
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Number,
    Bool,
    String,
    Range,
    List,
    Function,
    Class,
    Object,
    Null,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Type::Number => "number",
                Type::Bool => "bool",
                Type::String => "string",
                Type::Range => "range",
                Type::List => "list",
                Type::Function => "function",
                Type::Class => "class",
                Type::Object => "object",
                Type::Null => "null",
            }
        )
    }
}

pub fn type_of(value: &Value) -> Type {
    match value {
        Value::Number(_) => Type::Number,
        Value::Bool(_) => Type::Bool,
        Value::String(_) => Type::String,
        Value::Range { .. } => Type::Range,
        Value::List(_) => Type::List,
        Value::Function { .. } | Value::BuiltIn(_) => Type::Function,
        Value::Class { .. } => Type::Class,
        Value::Object(_) => Type::Object,
        Value::Null => Type::Null,
    }
}
