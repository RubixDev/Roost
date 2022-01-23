use bigdecimal::{BigDecimal, ToPrimitive};
use num_bigint::BigInt;

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
                Value::Int(val2) => { return Value::Bool(val1 < val2); },
                Value::Long(val2) => { return Value::Bool(&BigInt::from(*val1) < val2); },
                Value::Float(val2) => { return Value::Bool(&(*val1 as f64) < val2); },
                Value::Decimal(val2) => { return Value::Bool(&BigDecimal::from(*val1) < val2); },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            Value::Long(val1) => match other {
                Value::Int(val2) => { return Value::Bool(val1 < &BigInt::from(*val2)); },
                Value::Long(val2) => { return Value::Bool(val1 < val2); },
                Value::Float(val2) => {
                    if let Ok(val2) = BigDecimal::try_from(*val2) { return Value::Bool(BigDecimal::from(val1.clone()) < val2); }
                    return Value::Bool(&val1.to_f64().unwrap() < val2);
                },
                Value::Decimal(val2) => { return Value::Bool(&BigDecimal::from(val1.clone()) < val2); },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            Value::Float(val1) => match other {
                Value::Int(val2) => { return Value::Bool(val1 < &(*val2 as f64)); },
                Value::Long(val2) => {
                    if let Ok(val1) = BigDecimal::try_from(*val1) { return Value::Bool(val1 < BigDecimal::from(val2.clone())); }
                    return Value::Bool(val1 < &val2.to_f64().unwrap());
                },
                Value::Float(val2) => { return Value::Bool(val1 < val2); },
                Value::Decimal(val2) => {
                    if let Ok(val1) = BigDecimal::try_from(*val1) { return Value::Bool(&val1 < val2); }
                    return Value::Bool(val1 < &val2.to_f64().unwrap());
                },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            Value::Decimal(val1) => match other {
                Value::Int(val2) => { return Value::Bool(val1 < &BigDecimal::from(*val2)); },
                Value::Long(val2) => { return Value::Bool(val1 < &BigDecimal::from(val2.clone())); },
                Value::Float(val2) => {
                    if let Ok(val2) = BigDecimal::try_from(*val2) { return Value::Bool(val1 < &val2); }
                    return Value::Bool(&val1.to_f64().unwrap() < val2);
                },
                Value::Decimal(val2) => { return Value::Bool(val1 < val2); },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
        }
    }

    fn greater_than(&self, other: &Self) -> Self {
        match self {
            Value::Int(val1) => match other {
                Value::Int(val2) => { return Value::Bool(val1 > val2); },
                Value::Long(val2) => { return Value::Bool(&BigInt::from(*val1) > val2); },
                Value::Float(val2) => { return Value::Bool(&(*val1 as f64) > val2); },
                Value::Decimal(val2) => { return Value::Bool(&BigDecimal::from(*val1) > val2); },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            Value::Long(val1) => match other {
                Value::Int(val2) => { return Value::Bool(val1 > &BigInt::from(*val2)); },
                Value::Long(val2) => { return Value::Bool(val1 > val2); },
                Value::Float(val2) => {
                    if let Ok(val2) = BigDecimal::try_from(*val2) { return Value::Bool(BigDecimal::from(val1.clone()) > val2); }
                    return Value::Bool(&val1.to_f64().unwrap() > val2);
                },
                Value::Decimal(val2) => { return Value::Bool(&BigDecimal::from(val1.clone()) > val2); },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            Value::Float(val1) => match other {
                Value::Int(val2) => { return Value::Bool(val1 > &(*val2 as f64)); },
                Value::Long(val2) => {
                    if let Ok(val1) = BigDecimal::try_from(*val1) { return Value::Bool(val1 > BigDecimal::from(val2.clone())); }
                    return Value::Bool(val1 > &val2.to_f64().unwrap());
                },
                Value::Float(val2) => { return Value::Bool(val1 > val2); },
                Value::Decimal(val2) => {
                    if let Ok(val1) = BigDecimal::try_from(*val1) { return Value::Bool(&val1 > val2); }
                    return Value::Bool(val1 > &val2.to_f64().unwrap());
                },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            Value::Decimal(val1) => match other {
                Value::Int(val2) => { return Value::Bool(val1 > &BigDecimal::from(*val2)); },
                Value::Long(val2) => { return Value::Bool(val1 > &BigDecimal::from(val2.clone())); },
                Value::Float(val2) => {
                    if let Ok(val2) = BigDecimal::try_from(*val2) { return Value::Bool(val1 > &val2); }
                    return Value::Bool(&val1.to_f64().unwrap() > val2);
                },
                Value::Decimal(val2) => { return Value::Bool(val1 > val2); },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
        }
    }

    fn less_than_or_equal(&self, other: &Self) -> Self {
        match self {
            Value::Int(val1) => match other {
                Value::Int(val2) => { return Value::Bool(val1 <= val2); },
                Value::Long(val2) => { return Value::Bool(&BigInt::from(*val1) <= val2); },
                Value::Float(val2) => { return Value::Bool(&(*val1 as f64) <= val2); },
                Value::Decimal(val2) => { return Value::Bool(&BigDecimal::from(*val1) <= val2); },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            Value::Long(val1) => match other {
                Value::Int(val2) => { return Value::Bool(val1 <= &BigInt::from(*val2)); },
                Value::Long(val2) => { return Value::Bool(val1 <= val2); },
                Value::Float(val2) => {
                    if let Ok(val2) = BigDecimal::try_from(*val2) { return Value::Bool(BigDecimal::from(val1.clone()) <= val2); }
                    return Value::Bool(&val1.to_f64().unwrap() <= val2);
                },
                Value::Decimal(val2) => { return Value::Bool(&BigDecimal::from(val1.clone()) <= val2); },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            Value::Float(val1) => match other {
                Value::Int(val2) => { return Value::Bool(val1 <= &(*val2 as f64)); },
                Value::Long(val2) => {
                    if let Ok(val1) = BigDecimal::try_from(*val1) { return Value::Bool(val1 <= BigDecimal::from(val2.clone())); }
                    return Value::Bool(val1 <= &val2.to_f64().unwrap());
                },
                Value::Float(val2) => { return Value::Bool(val1 <= val2); },
                Value::Decimal(val2) => {
                    if let Ok(val1) = BigDecimal::try_from(*val1) { return Value::Bool(&val1 <= val2); }
                    return Value::Bool(val1 <= &val2.to_f64().unwrap());
                },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            Value::Decimal(val1) => match other {
                Value::Int(val2) => { return Value::Bool(val1 <= &BigDecimal::from(*val2)); },
                Value::Long(val2) => { return Value::Bool(val1 <= &BigDecimal::from(val2.clone())); },
                Value::Float(val2) => {
                    if let Ok(val2) = BigDecimal::try_from(*val2) { return Value::Bool(val1 <= &val2); }
                    return Value::Bool(&val1.to_f64().unwrap() <= val2);
                },
                Value::Decimal(val2) => { return Value::Bool(val1 <= val2); },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
        }
    }

    fn greater_than_or_equal(&self, other: &Self) -> Self {
        match self {
            Value::Int(val1) => match other {
                Value::Int(val2) => { return Value::Bool(val1 >= val2); },
                Value::Long(val2) => { return Value::Bool(&BigInt::from(*val1) >= val2); },
                Value::Float(val2) => { return Value::Bool(&(*val1 as f64) >= val2); },
                Value::Decimal(val2) => { return Value::Bool(&BigDecimal::from(*val1) >= val2); },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            Value::Long(val1) => match other {
                Value::Int(val2) => { return Value::Bool(val1 >= &BigInt::from(*val2)); },
                Value::Long(val2) => { return Value::Bool(val1 >= val2); },
                Value::Float(val2) => {
                    if let Ok(val2) = BigDecimal::try_from(*val2) { return Value::Bool(BigDecimal::from(val1.clone()) >= val2); }
                    return Value::Bool(&val1.to_f64().unwrap() >= val2);
                },
                Value::Decimal(val2) => { return Value::Bool(&BigDecimal::from(val1.clone()) >= val2); },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            Value::Float(val1) => match other {
                Value::Int(val2) => { return Value::Bool(val1 >= &(*val2 as f64)); },
                Value::Long(val2) => {
                    if let Ok(val1) = BigDecimal::try_from(*val1) { return Value::Bool(val1 >= BigDecimal::from(val2.clone())); }
                    return Value::Bool(val1 >= &val2.to_f64().unwrap());
                },
                Value::Float(val2) => { return Value::Bool(val1 >= val2); },
                Value::Decimal(val2) => {
                    if let Ok(val1) = BigDecimal::try_from(*val1) { return Value::Bool(&val1 >= val2); }
                    return Value::Bool(val1 >= &val2.to_f64().unwrap());
                },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            Value::Decimal(val1) => match other {
                Value::Int(val2) => { return Value::Bool(val1 >= &BigDecimal::from(*val2)); },
                Value::Long(val2) => { return Value::Bool(val1 >= &BigDecimal::from(val2.clone())); },
                Value::Float(val2) => {
                    if let Ok(val2) = BigDecimal::try_from(*val2) { return Value::Bool(val1 >= &val2); }
                    return Value::Bool(&val1.to_f64().unwrap() >= val2);
                },
                Value::Decimal(val2) => { return Value::Bool(val1 >= val2); },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
        }
    }
}
