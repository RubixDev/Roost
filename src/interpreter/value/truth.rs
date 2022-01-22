use bigdecimal::{Zero, BigDecimal};
use num_bigint::BigInt;

use super::Value;

pub trait Truth {
    fn is_true(&self) -> bool;
    fn is_false(&self) -> bool { !self.is_true() }
}

impl Truth for Value {
    fn is_true(&self) -> bool {
        match self {
            Value::Int(value) => value != &BigInt::zero(),
            Value::Float(value) => value != &BigDecimal::zero(),
            Value::Bool(value) => *value,
            Value::String(value) => !value.is_empty(),
            Value::Range(_, start, end) => start != end,
            Value::Null
            | Value::Void => false,
            _ => true,
        }
    }
}
