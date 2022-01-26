use rust_decimal::{MathematicalOps, Decimal, prelude::ToPrimitive};
use super::{Value, types::type_of};

pub trait CalculativeOperations {
    fn plus(&self, other: &Self) -> Self;
    fn minus(&self, other: &Self) -> Self;
    fn multiply(&self, other: &Self) -> Self;
    fn divide(&self, other: &Self) -> Self;
    fn power(&self, other: &Self) -> Self;
}

impl CalculativeOperations for Value {
    fn plus(&self, other: &Value) -> Value {
        match self {
            Value::Number(val1) => {
                match other {
                    Value::Number(val2) => { return Value::Number((val1 + val2).normalize()); },
                    Value::String(val2) => { return Value::String(self.to_string() + val2); },
                    _ => panic!("TypeError at position {{}}: Cannot add {} to {}", type_of(self), type_of(other)),
                }
            },
            Value::String(val1) => {
                return Value::String(val1.to_owned() + &other.to_string());
            },
            _ => {
                if let Value::String(val2) = other { return Value::String(self.to_string() + val2); }
                panic!("TypeError at position {{}}: Cannot add {} to {}", type_of(self), type_of(other));
            },
        }
    }

    fn minus(&self, other: &Value) -> Value {
        match self {
            Value::Number(val1) => {
                match other {
                    Value::Number(val2) => { return Value::Number((val1 - val2).normalize()); },
                    _ => panic!("TypeError at position {{}}: Cannot subtract {} from {}", type_of(other), type_of(self)),
                }
            },
            _ => panic!("TypeError at position {{}}: Cannot subtract {} from {}", type_of(other), type_of(self)),
        }
    }

    fn multiply(&self, other: &Value) -> Value {
        match self {
            Value::Number(val1) => {
                match other {
                    Value::Number(val2) => { return Value::Number((val1 * val2).normalize()); },
                    _ => { return other.multiply(self) },
                }
            },
            Value::String(val1) => {
                match other {
                    Value::Number(val2) => {
                        if !val2.fract().is_zero() {
                            panic!("ValueError at position {{}}: Cannot multiply string with fractional number");
                        }
                        if val2 < &Decimal::ZERO {
                            panic!("ValueError at position {{}}: Cannot multiply string with negative number");
                        }
                        let mut str = String::new();
                        let mut i = val2.to_i128().unwrap();
                        while i > 0 {
                            str += val1;
                            i -= 1;
                        }
                        return Value::String(str);
                    },
                    _ => panic!("TypeError at position {{}}: Cannot multiply {} with {}", type_of(self), type_of(other)),
                }
            },
            _ => {
                if let Value::String(_) = other { return other.multiply(self); }
                panic!("TypeError at position {{}}: Cannot multiply {} with {}", type_of(self), type_of(other));
            },
        }
    }

    fn divide(&self, other: &Value) -> Value {
        match self {
            Value::Number(val1) => {
                match other {
                    Value::Number(val2) => {
                        if val2.is_zero() { panic!("DivisionByZeroError at position {{}}: Cannot divide by zero"); }
                        return Value::Number((val1 / val2).normalize());
                    },
                    _ => panic!("TypeError at position {{}}: Cannot divide {} by {}", type_of(self), type_of(other)),
                }
            },
            _ => panic!("TypeError at position {{}}: Cannot divide {} by {}", type_of(self), type_of(other)),
        }
    }

    fn power(&self, other: &Self) -> Self {
        match self {
            Value::Number(val1) => {
                match other {
                    Value::Number(val2) => { return Value::Number(val1.powd(*val2).normalize()) },
                    _ => panic!("TypeError at position {{}}: Cannot raise {} by {}", type_of(self), type_of(other)),
                }
            },
            _ => panic!("TypeError at position {{}}: Cannot raise {} by {}", type_of(self), type_of(other)),
        }
    }
}
