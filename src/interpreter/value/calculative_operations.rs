use rust_decimal::{MathematicalOps, Decimal, prelude::ToPrimitive};
use super::{Value, types::type_of};
use crate::error::{Result, Location};

pub trait CalculativeOperations {
    fn plus(&self, other: &Value, start_loc: Location, end_loc: Location) -> Result<Value>;
    fn minus(&self, other: &Value, start_loc: Location, end_loc: Location) -> Result<Value>;
    fn multiply(&self, other: &Value, start_loc: Location, end_loc: Location) -> Result<Value>;
    fn divide(&self, other: &Value, start_loc: Location, end_loc: Location) -> Result<Value>;
    fn power(&self, other: &Value, start_loc: Location, end_loc: Location) -> Result<Value>;
    fn modulo(&self, other: &Value, start_loc: Location, end_loc: Location) -> Result<Value>;
    fn int_divide(&self, other: &Value, start_loc: Location, end_loc: Location) -> Result<Value>;
}

impl CalculativeOperations for Value {
    fn plus(&self, other: &Value, start_loc: Location, end_loc: Location) -> Result<Value> {
        match self {
            Value::Number(val1) => {
                match other {
                    Value::Number(val2) => { return Ok(Value::Number((match val1.checked_add(*val2) {
                        Some(result) => result,
                        None => error!(OverflowError, start_loc, end_loc, "Addition resulted in overflow"),
                    }).normalize())); },
                    Value::String(val2) => { return Ok(Value::String(self.to_string() + val2)); },
                    _ => error!(TypeError, start_loc, end_loc, "Cannot add {} to {}", type_of(self), type_of(other)),
                }
            },
            Value::String(val1) => {
                return Ok(Value::String(val1.to_owned() + &other.to_string()));
            },
            _ => {
                if let Value::String(val2) = other { return Ok(Value::String(self.to_string() + val2)); }
                error!(TypeError, start_loc, end_loc, "Cannot add {} to {}", type_of(self), type_of(other));
            },
        }
    }

    fn minus(&self, other: &Value, start_loc: Location, end_loc: Location) -> Result<Value> {
        match self {
            Value::Number(val1) => {
                match other {
                    Value::Number(val2) => { return Ok(Value::Number((match val1.checked_sub(*val2) {
                        Some(result) => result,
                        None => error!(OverflowError, start_loc, end_loc, "Subtraction resulted in overflow"),
                    }).normalize())); },
                    _ => error!(TypeError, start_loc, end_loc, "Cannot subtract {} from {}", type_of(other), type_of(self)),
                }
            },
            _ => error!(TypeError, start_loc, end_loc, "Cannot subtract {} from {}", type_of(other), type_of(self)),
        }
    }

    fn multiply(&self, other: &Value, start_loc: Location, end_loc: Location) -> Result<Value> {
        match self {
            Value::Number(val1) => {
                match other {
                    Value::Number(val2) => { return Ok(Value::Number((match val1.checked_mul(*val2) {
                        Some(result) => result,
                        None => error!(OverflowError, start_loc, end_loc, "Multiplication resulted in overflow"),
                    }).normalize())); },
                    _ => { return other.multiply(self, start_loc, end_loc) },
                }
            },
            Value::String(val1) => {
                match other {
                    Value::Number(val2) => {
                        if !val2.fract().is_zero() {
                            error!(ValueError, start_loc, end_loc, "Cannot multiply string with fractional number");
                        }
                        if val2 < &Decimal::ZERO {
                            error!(ValueError, start_loc, end_loc, "Cannot multiply string with negative number");
                        }
                        let mut str = String::new();
                        let mut i = val2.to_i128().unwrap();
                        while i > 0 {
                            str += val1;
                            i -= 1;
                        }
                        return Ok(Value::String(str));
                    },
                    _ => error!(TypeError, start_loc, end_loc, "Cannot multiply {} with {}", type_of(self), type_of(other)),
                }
            },
            _ => {
                if let Value::String(_) = other { return other.multiply(self, start_loc, end_loc); }
                error!(TypeError, start_loc, end_loc, "Cannot multiply {} with {}", type_of(self), type_of(other));
            },
        }
    }

    fn divide(&self, other: &Value, start_loc: Location, end_loc: Location) -> Result<Value> {
        match self {
            Value::Number(val1) => {
                match other {
                    Value::Number(val2) => {
                        if val2.is_zero() { error!(DivisionByZeroError, start_loc, end_loc, "Cannot divide by zero"); }
                        return Ok(Value::Number((match val1.checked_div(*val2) {
                            Some(result) => result,
                            None => error!(OverflowError, start_loc, end_loc, "Division resulted in overflow"),
                        }).normalize()));
                    },
                    _ => error!(TypeError, start_loc, end_loc, "Cannot divide {} by {}", type_of(self), type_of(other)),
                }
            },
            _ => error!(TypeError, start_loc, end_loc, "Cannot divide {} by {}", type_of(self), type_of(other)),
        }
    }

    fn power(&self, other: &Self, start_loc: Location, end_loc: Location) -> Result<Value> {
        match self {
            Value::Number(val1) => {
                match other {
                    Value::Number(val2) => { return Ok(Value::Number(match val1.checked_powd(*val2) {
                        Some(result) => result,
                        None => error!(OverflowError, start_loc, end_loc, "Power resulted in overflow"),
                    }.normalize())) },
                    _ => error!(TypeError, start_loc, end_loc, "Cannot raise {} by {}", type_of(self), type_of(other)),
                }
            },
            _ => error!(TypeError, start_loc, end_loc, "Cannot raise {} by {}", type_of(self), type_of(other)),
        }
    }

    fn modulo(&self, other: &Self, start_loc: Location, end_loc: Location) -> Result<Value> {
        match self {
            Value::Number(val1) => {
                match other {
                    Value::Number(val2) => {
                        if val2.is_zero() { error!(DivisionByZeroError, start_loc, end_loc, "Cannot divide by zero"); }
                        return Ok(Value::Number((val1 % val2).normalize()))
                    },
                    _ => error!(TypeError, start_loc, end_loc, "Cannot raise {} by {}", type_of(self), type_of(other)),
                }
            },
            _ => error!(TypeError, start_loc, end_loc, "Cannot raise {} by {}", type_of(self), type_of(other)),
        }
    }

    fn int_divide(&self, other: &Value, start_loc: Location, end_loc: Location) -> Result<Value> {
        match self {
            Value::Number(val1) => {
                match other {
                    Value::Number(val2) => {
                        if val2.is_zero() { error!(DivisionByZeroError, start_loc, end_loc, "Cannot divide by zero"); }
                        return Ok(Value::Number((match val1.checked_div(*val2) {
                            Some(result) => result,
                            None => error!(OverflowError, start_loc, end_loc, "Division resulted in overflow"),
                        }).trunc().normalize()));
                    },
                    _ => error!(TypeError, start_loc, end_loc, "Cannot divide {} by {}", type_of(self), type_of(other)),
                }
            },
            _ => error!(TypeError, start_loc, end_loc, "Cannot divide {} by {}", type_of(self), type_of(other)),
        }
    }
}
