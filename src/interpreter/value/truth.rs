use bigdecimal::{Zero, BigDecimal};
use num_bigint::BigInt;

use super::{Value, Range};

pub trait Truth {
    fn is_true(&self) -> bool;
    fn is_false(&self) -> bool { !self.is_true() }
}

impl Truth for Value {
    fn is_true(&self) -> bool {
        match self {
            Value::Long(value) => value != &BigInt::zero(),
            Value::Decimal(value) => value != &BigDecimal::zero(),
            Value::Bool(value) => *value,
            Value::String(value) => !value.is_empty(),
            Value::Range(value) => match value {
                Range::Int(_, start, end) => start != end,
                Range::Long(_, start, end) => start != end,
            },
            Value::Null
            | Value::Void => false,
            _ => true,
        }
    }
}
