use bigdecimal::{BigDecimal, Zero};
use num_bigint::BigInt;
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
            Value::Int(val1) => {
                match other {
                    Value::Int(val2) => {
                        return Value::Int(val1 + val2);
                    },
                    Value::Float(val2) => {
                        return Value::Float(BigDecimal::from(val1.clone()) + val2);
                    },
                    Value::String(val2) => {
                        return Value::String(val1.to_string() + &val2);
                    },
                    _ => panic!(
                        "TypeError at position {{}}: Cannot add {} to int",
                        type_of(other),
                    ),
                }
            },
            Value::Float(val1) => {
                match other {
                    Value::Int(val2) => {
                        return Value::Float(val1 + BigDecimal::from(val2.clone()));
                    },
                    Value::Float(val2) => {
                        return Value::Float(val1 + val2);
                    },
                    Value::String(val2) => {
                        return Value::String(val1.to_string() + &val2);
                    },
                    _ => panic!(
                        "TypeError at position {{}}: Cannot add {} to float",
                        type_of(other),
                    ),
                }
            },
            Value::String(val1) => {
                return Value::String(val1.to_owned() + &other.to_string());
            },
            _ => panic!(
                "TypeError at position {{}}: Cannot add {} to {}",
                type_of(other),
                type_of(self),
            ),
        }
    }

    fn minus(&self, other: &Value) -> Value {
        match self {
            Value::Int(val1) => {
                match other {
                    Value::Int(val2) => {
                        return Value::Int(val1 - val2);
                    },
                    Value::Float(val2) => {
                        return Value::Float(BigDecimal::from(val1.clone()) - val2);
                    },
                    _ => panic!(
                        "TypeError at position {{}}: Cannot subtract {} from int",
                        type_of(other),
                    ),
                }
            },
            Value::Float(val1) => {
                match other {
                    Value::Int(val2) => {
                        return Value::Float(val1 - BigDecimal::from(val2.clone()));
                    },
                    Value::Float(val2) => {
                        return Value::Float(val1 - val2);
                    },
                    _ => panic!(
                        "TypeError at position {{}}: Cannot subtract {} from float",
                        type_of(other),
                    ),
                }
            },
            _ => panic!(
                "TypeError at position {{}}: Cannot subtract {} from {}",
                type_of(other),
                type_of(self),
            ),
        }
    }

    fn multiply(&self, other: &Value) -> Value {
        match self {
            Value::Int(val1) => {
                match other {
                    Value::Int(val2) => {
                        return Value::Int(val1 * val2);
                    },
                    Value::Float(val2) => {
                        return Value::Float(BigDecimal::from(val1.clone()) * val2);
                    },
                    _ => panic!(
                        "TypeError at position {{}}: Cannot multiply int with {}",
                        type_of(other),
                    ),
                }
            },
            Value::Float(val1) => {
                match other {
                    Value::Int(val2) => {
                        return Value::Float(val1 * BigDecimal::from(val2.clone()));
                    },
                    Value::Float(val2) => {
                        return Value::Float(val1 * val2);
                    },
                    _ => panic!(
                        "TypeError at position {{}}: Cannot multiply float with {}",
                        type_of(other),
                    ),
                }
            },
            Value::String(val1) => {
                match other {
                    Value::Int(val2) => {
                        let mut str = val1.clone();
                        let mut i = val2.clone();
                        while i > BigInt::zero() {
                            str += val1;
                            i -= 1u8;
                        }
                        return Value::String(str);
                    },
                    _ => panic!(
                        "TypeError at position {{}}: Cannot multiply float with {}",
                        type_of(other),
                    ),
                }
            },
            _ => panic!(
                "TypeError at position {{}}: Cannot multiply {} with {}",
                type_of(self),
                type_of(other),
            ),
        }
    }

    fn divide(&self, other: &Value) -> Value {
        match self {
            Value::Int(val1) => {
                match other {
                    Value::Int(val2) => {
                        return Value::Int(val1 / val2);
                    },
                    Value::Float(val2) => {
                        return Value::Float(BigDecimal::from(val1.clone()) / val2);
                    },
                    _ => panic!(
                        "TypeError at position {{}}: Cannot divide int by {}",
                        type_of(other),
                    ),
                }
            },
            Value::Float(val1) => {
                match other {
                    Value::Int(val2) => {
                        return Value::Float(val1 / BigDecimal::from(val2.clone()));
                    },
                    Value::Float(val2) => {
                        return Value::Float(val1 / val2);
                    },
                    _ => panic!(
                        "TypeError at position {{}}: Cannot divide float by {}",
                        type_of(other),
                    ),
                }
            },
            _ => panic!(
                "TypeError at position {{}}: Cannot divide {} by {}",
                type_of(self),
                type_of(other),
            ),
        }
    }

    fn power(&self, other: &Self) -> Self {
        match self {
            Value::Int(val1) => {
                match other {
                    Value::Int(val2) => {
                        let mut out = val1.clone();
                        let mut i = BigInt::zero() + 1;
                        while &i < val2 {
                            out *= val1;
                            i += 1;
                        }
                        return Value::Int(out);
                    },
                    // Value::Float(val2) => {
                    //     return Value::Float(BigDecimal::from(val1.clone()) / val2);
                    // },
                    _ => panic!(
                        "TypeError at position {{}}: Cannot raise int to {}",
                        type_of(other),
                    ),
                }
            },
            Value::Float(val1) => {
                match other {
                    Value::Int(val2) => {
                        let mut out = val1.clone();
                        let mut i = BigInt::zero() + 1;
                        while &i < val2 {
                            out *= val1;
                            i += 1;
                        }
                        return Value::Float(out);
                    },
                    // Value::Float(val2) => {
                    //     return Value::Float(val1 / val2);
                    // },
                    _ => panic!(
                        "TypeError at position {{}}: Cannot raise float to {}",
                        type_of(other),
                    ),
                }
            },
            _ => panic!(
                "TypeError at position {{}}: Cannot raise {} to {}",
                type_of(self),
                type_of(other),
            ),
        }
    }
}
