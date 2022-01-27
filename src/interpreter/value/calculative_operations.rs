use rust_decimal::{MathematicalOps, Decimal, prelude::ToPrimitive};
use super::{Value, types::type_of};
use crate::error::{Result, Location};

pub trait CalculativeOperations {
    fn plus(&self, other: &Value, location: Location) -> Result<Value>;
    fn minus(&self, other: &Value, location: Location) -> Result<Value>;
    fn multiply(&self, other: &Value, location: Location) -> Result<Value>;
    fn divide(&self, other: &Value, location: Location) -> Result<Value>;
    fn power(&self, other: &Value, location: Location) -> Result<Value>;
}

impl CalculativeOperations for Value {
    fn plus(&self, other: &Value, location: Location) -> Result<Value> {
        match self {
            Value::Number(val1) => {
                match other {
                    Value::Number(val2) => { return Ok(Value::Number((match val1.checked_add(*val2) {
                        Some(result) => result,
                        None => error!(OverflowError, location, "Addition resulted in overflow"),
                    }).normalize())); },
                    Value::String(val2) => { return Ok(Value::String(self.to_string() + val2)); },
                    _ => error!(TypeError, location, "Cannot add {} to {}", type_of(self), type_of(other)),
                }
            },
            Value::String(val1) => {
                return Ok(Value::String(val1.to_owned() + &other.to_string()));
            },
            _ => {
                if let Value::String(val2) = other { return Ok(Value::String(self.to_string() + val2)); }
                error!(TypeError, location, "Cannot add {} to {}", type_of(self), type_of(other));
            },
        }
    }

    fn minus(&self, other: &Value, location: Location) -> Result<Value> {
        match self {
            Value::Number(val1) => {
                match other {
                    Value::Number(val2) => { return Ok(Value::Number((match val1.checked_sub(*val2) {
                        Some(result) => result,
                        None => error!(OverflowError, location, "Subtraction resulted in overflow"),
                    }).normalize())); },
                    _ => error!(TypeError, location, "Cannot subtract {} from {}", type_of(other), type_of(self)),
                }
            },
            _ => error!(TypeError, location, "Cannot subtract {} from {}", type_of(other), type_of(self)),
        }
    }

    fn multiply(&self, other: &Value, location: Location) -> Result<Value> {
        match self {
            Value::Number(val1) => {
                match other {
                    Value::Number(val2) => { return Ok(Value::Number((match val1.checked_mul(*val2) {
                        Some(result) => result,
                        None => error!(OverflowError, location, "Multiplication resulted in overflow"),
                    }).normalize())); },
                    _ => { return other.multiply(self, location) },
                }
            },
            Value::String(val1) => {
                match other {
                    Value::Number(val2) => {
                        if !val2.fract().is_zero() {
                            error!(ValueError, location, "Cannot multiply string with fractional number");
                        }
                        if val2 < &Decimal::ZERO {
                            error!(ValueError, location, "Cannot multiply string with negative number");
                        }
                        let mut str = String::new();
                        let mut i = val2.to_i128().unwrap();
                        while i > 0 {
                            str += val1;
                            i -= 1;
                        }
                        return Ok(Value::String(str));
                    },
                    _ => error!(TypeError, location, "Cannot multiply {} with {}", type_of(self), type_of(other)),
                }
            },
            _ => {
                if let Value::String(_) = other { return other.multiply(self, location); }
                error!(TypeError, location, "Cannot multiply {} with {}", type_of(self), type_of(other));
            },
        }
    }

    fn divide(&self, other: &Value, location: Location) -> Result<Value> {
        match self {
            Value::Number(val1) => {
                match other {
                    Value::Number(val2) => {
                        if val2.is_zero() { error!(DivisionByZeroError, location, "Cannot divide by zero"); }
                        return Ok(Value::Number((match val1.checked_div(*val2) {
                            Some(result) => result,
                            None => error!(OverflowError, location, "Division resulted in overflow"),
                        }).normalize()));
                    },
                    _ => error!(TypeError, location, "Cannot divide {} by {}", type_of(self), type_of(other)),
                }
            },
            _ => error!(TypeError, location, "Cannot divide {} by {}", type_of(self), type_of(other)),
        }
    }

    fn power(&self, other: &Self, location: Location) -> Result<Value> {
        match self {
            Value::Number(val1) => {
                match other {
                    Value::Number(val2) => { return Ok(Value::Number(match val1.checked_powd(*val2) {
                        Some(result) => result,
                        None => error!(OverflowError, location, "Power resulted in overflow"),
                    }.normalize())) },
                    _ => error!(TypeError, location, "Cannot raise {} by {}", type_of(self), type_of(other)),
                }
            },
            _ => error!(TypeError, location, "Cannot raise {} by {}", type_of(self), type_of(other)),
        }
    }
}
