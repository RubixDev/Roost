use super::Value;

pub trait Truth {
    fn is_true(&self) -> bool;
    fn is_false(&self) -> bool { !self.is_true() }
}

impl Truth for Value {
    fn is_true(&self) -> bool {
        match self {
            Value::Number(value) => !value.is_zero(),
            Value::Bool(value) => *value,
            Value::String(value) => !value.is_empty(),
            Value::Range(start, end) => start != end,
            Value::Null
            | Value::Void => false,
            _ => true,
        }
    }
}
