use super::Value;

impl Value<'_> {
    pub fn is_true(&self) -> bool {
        match self {
            Value::Number(value) => !value.is_zero(),
            Value::Bool(value) => *value,
            Value::String(value) => !value.is_empty(),
            Value::Range { start, end } => start != end,
            Value::Null => false,
            _ => true,
        }
    }

    pub fn is_false(&self) -> bool {
        !self.is_true()
    }
}
