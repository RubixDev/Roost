use super::{Value, types::type_of};
use crate::error::{Result, Location};

pub trait RelationalOperations {
    fn less_than(&self, other: &Value, location: Location) -> Result<Value>;
    fn greater_than(&self, other: &Value, location: Location) -> Result<Value>;
    fn less_than_or_equal(&self, other: &Value, location: Location) -> Result<Value>;
    fn greater_than_or_equal(&self, other: &Value, location: Location) -> Result<Value>;
}

impl RelationalOperations for Value {
    fn less_than(&self, other: &Value, location: Location) -> Result<Value> {
        match self {
            Value::Number(val1) => match other {
                Value::Number(val2) => { return Ok(Value::Bool(val1 < val2)); },
                _ => error!(TypeError, location, "Cannot compare {} with {}", type_of(self), type_of(other)),
            },
            _ => error!(TypeError, location, "Cannot compare {} with {}", type_of(self), type_of(other)),
        }
    }

    fn greater_than(&self, other: &Value, location: Location) -> Result<Value> {
        match self {
            Value::Number(val1) => match other {
                Value::Number(val2) => { return Ok(Value::Bool(val1 > val2)); },
                _ => error!(TypeError, location, "Cannot compare {} with {}", type_of(self), type_of(other)),
            },
            _ => error!(TypeError, location, "Cannot compare {} with {}", type_of(self), type_of(other)),
        }
    }

    fn less_than_or_equal(&self, other: &Value, location: Location) -> Result<Value> {
        match self {
            Value::Number(val1) => match other {
                Value::Number(val2) => { return Ok(Value::Bool(val1 <= val2)); },
                _ => error!(TypeError, location, "Cannot compare {} with {}", type_of(self), type_of(other)),
            },
            _ => error!(TypeError, location, "Cannot compare {} with {}", type_of(self), type_of(other)),
        }
    }

    fn greater_than_or_equal(&self, other: &Value, location: Location) -> Result<Value> {
        match self {
            Value::Number(val1) => match other {
                Value::Number(val2) => { return Ok(Value::Bool(val1 >= val2)); },
                _ => error!(TypeError, location, "Cannot compare {} with {}", type_of(self), type_of(other)),
            },
            _ => error!(TypeError, location, "Cannot compare {} with {}", type_of(self), type_of(other)),
        }
    }
}
