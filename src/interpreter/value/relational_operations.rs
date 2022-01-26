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
            Value::Number(val1) => match other {
                Value::Number(val2) => { return Value::Bool(val1 < val2); },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
        }
    }

    fn greater_than(&self, other: &Self) -> Self {
        match self {
            Value::Number(val1) => match other {
                Value::Number(val2) => { return Value::Bool(val1 > val2); },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
        }
    }

    fn less_than_or_equal(&self, other: &Self) -> Self {
        match self {
            Value::Number(val1) => match other {
                Value::Number(val2) => { return Value::Bool(val1 <= val2); },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
        }
    }

    fn greater_than_or_equal(&self, other: &Self) -> Self {
        match self {
            Value::Number(val1) => match other {
                Value::Number(val2) => { return Value::Bool(val1 >= val2); },
                _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
            },
            _ => panic!( "TypeError at position {{}}: cannot compare {} with {}", type_of(self), type_of(other)),
        }
    }
}
