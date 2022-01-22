use bigdecimal::BigDecimal;

use super::{Value, types::type_of};

pub trait RelationalOperations {
    fn less_than(&self, other: &Self) -> Self;
    fn greater_than(&self, other: &Self) -> Self;
    fn less_than_or_equal(&self, other: &Self) -> Self;
    fn greater_than_or_equal(&self, other: &Self) -> Self;
}

impl RelationalOperations for Value {
    fn less_than(&self, other: &Self) -> Self {
        match self {
            Value::Int(val1) => match other {
                Value::Int(val2) => {
                    return Value::Bool(val1 < val2);
                },
                Value::Float(val2) => {
                    return Value::Bool(&BigDecimal::from(val1.clone()) < val2);
                },
                _ => panic!(
                    "TypeError at position {{}}: cannot compare {} with {}",
                    type_of(self),
                    type_of(other),
                ),
            },
            Value::Float(val1) => match other {
                Value::Int(val2) => {
                    return Value::Bool(val1 < &BigDecimal::from(val2.clone()));
                },
                Value::Float(val2) => {
                    return Value::Bool(val1 < val2);
                },
                _ => panic!(
                    "TypeError at position {{}}: cannot compare {} with {}",
                    type_of(self),
                    type_of(other),
                ),
            },
            _ => panic!(
                "TypeError at position {{}}: cannot compare {} with {}",
                type_of(self),
                type_of(other),
            ),
        }
    }

    fn greater_than(&self, other: &Self) -> Self {
        match self {
            Value::Int(val1) => match other {
                Value::Int(val2) => {
                    return Value::Bool(val1 > val2);
                },
                Value::Float(val2) => {
                    return Value::Bool(&BigDecimal::from(val1.clone()) > val2);
                },
                _ => panic!(
                    "TypeError at position {{}}: cannot compare {} with {}",
                    type_of(self),
                    type_of(other),
                ),
            },
            Value::Float(val1) => match other {
                Value::Int(val2) => {
                    return Value::Bool(val1 > &BigDecimal::from(val2.clone()));
                },
                Value::Float(val2) => {
                    return Value::Bool(val1 > val2);
                },
                _ => panic!(
                    "TypeError at position {{}}: cannot compare {} with {}",
                    type_of(self),
                    type_of(other),
                ),
            },
            _ => panic!(
                "TypeError at position {{}}: cannot compare {} with {}",
                type_of(self),
                type_of(other),
            ),
        }
    }

    fn less_than_or_equal(&self, other: &Self) -> Self {
        match self {
            Value::Int(val1) => match other {
                Value::Int(val2) => {
                    return Value::Bool(val1 <= val2);
                },
                Value::Float(val2) => {
                    return Value::Bool(&BigDecimal::from(val1.clone()) <= val2);
                },
                _ => panic!(
                    "TypeError at position {{}}: cannot compare {} with {}",
                    type_of(self),
                    type_of(other),
                ),
            },
            Value::Float(val1) => match other {
                Value::Int(val2) => {
                    return Value::Bool(val1 <= &BigDecimal::from(val2.clone()));
                },
                Value::Float(val2) => {
                    return Value::Bool(val1 <= val2);
                },
                _ => panic!(
                    "TypeError at position {{}}: cannot compare {} with {}",
                    type_of(self),
                    type_of(other),
                ),
            },
            _ => panic!(
                "TypeError at position {{}}: cannot compare {} with {}",
                type_of(self),
                type_of(other),
            ),
        }
    }

    fn greater_than_or_equal(&self, other: &Self) -> Self {
        match self {
            Value::Int(val1) => match other {
                Value::Int(val2) => {
                    return Value::Bool(val1 >= val2);
                },
                Value::Float(val2) => {
                    return Value::Bool(&BigDecimal::from(val1.clone()) >= val2);
                },
                _ => panic!(
                    "TypeError at position {{}}: cannot compare {} with {}",
                    type_of(self),
                    type_of(other),
                ),
            },
            Value::Float(val1) => match other {
                Value::Int(val2) => {
                    return Value::Bool(val1 >= &BigDecimal::from(val2.clone()));
                },
                Value::Float(val2) => {
                    return Value::Bool(val1 >= val2);
                },
                _ => panic!(
                    "TypeError at position {{}}: cannot compare {} with {}",
                    type_of(self),
                    type_of(other),
                ),
            },
            _ => panic!(
                "TypeError at position {{}}: cannot compare {} with {}",
                type_of(self),
                type_of(other),
            ),
        }
    }
}
