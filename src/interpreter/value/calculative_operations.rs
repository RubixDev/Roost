use bigdecimal::{BigDecimal, Zero, ToPrimitive, num_traits::Pow};
use num_bigint::BigInt;
use super::{Value, types::type_of};

fn long_to_f64(long: &BigInt) -> Option<f64> {
    let new = long.to_f64()?;
    if new.is_infinite() { None } else { Some(new) }
}

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
                        let new = val1.checked_add(*val2);
                        if let Some(new) = new { return Value::Int(new); }
                        return Value::Long(BigInt::from(*val1) + val2);
                    },
                    Value::Long(val2) => {
                        if let Some(val2) = val2.to_i64() {
                            return self.plus(&Value::Int(val2));
                        }
                        let new = val1 + val2;
                        if let Some(new) = new.to_i64() { return Value::Int(new); }
                        return Value::Long(new);
                    },
                    Value::Float(val2) => { return Value::Float((*val1 as f64) + val2); },
                    Value::Decimal(val2) => { return Value::Decimal(val2 + BigInt::from(*val1)); },
                    Value::String(val2) => { return Value::String(self.to_string() + val2); },
                    _ => { return other.plus(self) },
                }
            },
            Value::Long(val1) => {
                match other {
                    Value::Long(val2) => { return Value::Long(val1 + val2); },
                    Value::Float(val2) => {
                        if let Some(val1) = long_to_f64(val1) {
                            return Value::Float(val1 + val2);
                        }
                        if let Ok(val2) = BigDecimal::try_from(*val2) {
                            let new = val2 + val1;
                            if let Some(new) = new.to_f64() { return Value::Float(new); }
                            return Value::Decimal(new);
                        }
                        return Value::Float(val1.to_f64().unwrap() + val2);
                    },
                    Value::Decimal(val2) => { return Value::Decimal(val2 + val1); },
                    Value::String(val2) => { return Value::String(self.to_string() + val2); },
                    _ => { return other.plus(self) },
                }
            },
            Value::Float(val1) => {
                match other {
                    Value::Float(val2) => { return Value::Float(val1 + val2); },
                    Value::Decimal(val2) => {
                        if let Ok(val1) = BigDecimal::try_from(*val1) { return Value::Decimal(val1 + val2); }
                        return Value::Float(val1 + val2.to_f64().unwrap());
                    },
                    Value::String(val2) => { return Value::String(self.to_string() + val2); },
                    _ => { return other.plus(self) },
                }
            }
            Value::Decimal(val1) => {
                match other {
                    Value::Decimal(val2) => { return Value::Decimal(val1 + val2); },
                    Value::String(val2) => { return Value::String(self.to_string() + val2); },
                    _ => { return other.plus(self) },
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
            Value::Int(val1) => {
                match other {
                    Value::Int(val2) => {
                        let new = val1.checked_sub(*val2);
                        if let Some(new) = new { return Value::Int(new); }
                        return Value::Long(BigInt::from(*val1) - val2);
                    },
                    Value::Long(val2) => {
                        if let Some(val2) = val2.to_i64() {
                            return self.minus(&Value::Int(val2));
                        }
                        let new = val1 - val2;
                        if let Some(new) = new.to_i64() { return Value::Int(new); }
                        return Value::Long(new);
                    },
                    Value::Float(val2) => { return Value::Float((*val1 as f64) - val2); },
                    Value::Decimal(val2) => { return Value::Decimal(BigDecimal::from(*val1) - val2); },
                    _ => panic!("TypeError at position {{}}: Cannot subtract {} from {}", type_of(other), type_of(self)),
                }
            },
            Value::Long(val1) => {
                match other {
                    Value::Int(val2) => {
                        if let Some(val1) = val1.to_i64() {
                            return Value::Int(val1).minus(other);
                        }
                        let new = val1 - val2;
                        if let Some(new) = new.to_i64() { return Value::Int(new); }
                        return Value::Long(new);
                    },
                    Value::Long(val2) => { return Value::Long(val1 - val2); },
                    Value::Float(val2) => {
                        if let Some(val1) = long_to_f64(val1) {
                            return Value::Float(val1 - val2);
                        }
                        if let Ok(val2) = BigDecimal::try_from(*val2) {
                            let new = BigDecimal::from(val1.clone()) - val2;
                            if let Some(new) = new.to_f64() { return Value::Float(new); }
                            return Value::Decimal(new);
                        }
                        return Value::Float(val1.to_f64().unwrap() - val2);
                    },
                    Value::Decimal(val2) => { return Value::Decimal(BigDecimal::from(val1.clone()) - val2); },
                    _ => panic!("TypeError at position {{}}: Cannot subtract {} from {}", type_of(other), type_of(self)),
                }
            },
            Value::Float(val1) => {
                match other {
                    Value::Int(val2) => { return Value::Float(val1 - (*val2 as f64)); },
                    Value::Long(val2) => {
                        if let Some(val2) = long_to_f64(val2) {
                            return Value::Float(val1 - val2);
                        }
                        if let Ok(val1) = BigDecimal::try_from(*val1) {
                            let new = val1 - val2;
                            if let Some(new) = new.to_f64() { return Value::Float(new); }
                            return Value::Decimal(new);
                        }
                        return Value::Float(val1 - val2.to_f64().unwrap());
                    },
                    Value::Float(val2) => { return Value::Float(val1 - val2); },
                    Value::Decimal(val2) => {
                        if let Ok(val1) = BigDecimal::try_from(*val1) { return Value::Decimal(val1 - val2); }
                        return Value::Float(val1 - val2.to_f64().unwrap());
                    },
                    _ => panic!("TypeError at position {{}}: Cannot subtract {} from {}", type_of(other), type_of(self)),
                }
            },
            Value::Decimal(val1) => {
                match other {
                    Value::Int(val2) => { return Value::Decimal(val1 - BigInt::from(*val2)); },
                    Value::Long(val2) => { return Value::Decimal(val1 - val2); },
                    Value::Float(val2) => {
                        if let Ok(val2) = BigDecimal::try_from(*val2) { return Value::Decimal(val1 - val2); }
                        return Value::Float(val1.to_f64().unwrap() - val2);
                    },
                    Value::Decimal(val2) => { return Value::Decimal(val1 - val2); },
                    _ => panic!("TypeError at position {{}}: Cannot subtract {} from {}", type_of(other), type_of(self)),
                }
            },
            _ => panic!("TypeError at position {{}}: Cannot subtract {} from {}", type_of(other), type_of(self)),
        }
    }

    fn multiply(&self, other: &Value) -> Value {
        match self {
            Value::Int(val1) => {
                match other {
                    Value::Int(val2) => {
                        let new = val1.checked_mul(*val2);
                        if let Some(new) = new { return Value::Int(new); }
                        return Value::Long(BigInt::from(*val1) * val2);
                    },
                    Value::Long(val2) => {
                        if let Some(val2) = val2.to_i64() {
                            return self.multiply(&Value::Int(val2));
                        }
                        let new = val1 * val2;
                        if let Some(new) = new.to_i64() { return Value::Int(new); }
                        return Value::Long(new);
                    },
                    Value::Float(val2) => { return Value::Float((*val1 as f64) * val2); },
                    Value::Decimal(val2) => { return Value::Decimal(val2 * BigInt::from(*val1)); },
                    _ => { return other.multiply(self) },
                }
            },
            Value::Long(val1) => {
                match other {
                    Value::Long(val2) => { return Value::Long(val1 * val2); },
                    Value::Float(val2) => {
                        if let Some(val1) = long_to_f64(val1) {
                            return Value::Float(val1 * val2);
                        }
                        if let Ok(val2) = BigDecimal::try_from(*val2) {
                            let new = val2 * val1;
                            if let Some(new) = new.to_f64() { return Value::Float(new); }
                            return Value::Decimal(new);
                        }
                        return Value::Float(val1.to_f64().unwrap() * val2);
                    },
                    Value::Decimal(val2) => { return Value::Decimal(val2 * val1); },
                    _ => { return other.multiply(self) },
                }
            },
            Value::Float(val1) => {
                match other {
                    Value::Float(val2) => { return Value::Float(val1 * val2); },
                    Value::Decimal(val2) => {
                        if let Ok(val1) = BigDecimal::try_from(*val1) { return Value::Decimal(val1 * val2); }
                        return Value::Float(val1 * val2.to_f64().unwrap());
                    },
                    _ => { return other.multiply(self) },
                }
            },
            Value::Decimal(val1) => {
                match other {
                    Value::Decimal(val2) => { return Value::Decimal(val1 * val2); },
                    _ => { return other.multiply(self) },
                }
            },
            Value::String(val1) => {
                match other {
                    Value::Int(val2) => { return Value::String(val1.repeat(val2.to_usize().unwrap_or(0))); },
                    Value::Long(val2) => {
                        if val2 < &BigInt::zero() { return Value::String(String::new()); }
                        let mut str = val1.clone();
                        let mut i = val2.clone();
                        while i > BigInt::zero() {
                            str += val1;
                            i -= 1u8;
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
            Value::Int(val1) => {
                match other {
                    Value::Int(val2) => {
                        if val2.is_zero() { panic!("DivisionByZeroError at position {{}}: Cannot divide by zero"); }
                        return Value::Float(*val1 as f64 / *val2 as f64);
                    },
                    Value::Long(val2) => {
                        if val2.is_zero() { panic!("DivisionByZeroError at position {{}}: Cannot divide by zero"); }
                        if let Some(val2) = val2.to_i64() { return self.divide(&Value::Int(val2)); }
                        if let Some(val2) = long_to_f64(val2) { return Value::Float(*val1 as f64 / val2); }
                        let new = BigDecimal::from(*val1) / BigDecimal::from(val2.clone());
                        if let Some(new) = new.to_f64() { return Value::Float(new); }
                        return Value::Decimal(new);
                    },
                    Value::Float(val2) => {
                        if val2.is_zero() { panic!("DivisionByZeroError at position {{}}: Cannot divide by zero"); }
                        return Value::Float((*val1 as f64) / val2);
                    },
                    Value::Decimal(val2) => {
                        if val2.is_zero() { panic!("DivisionByZeroError at position {{}}: Cannot divide by zero"); }
                        return Value::Decimal(BigDecimal::from(*val1) / val2);
                    },
                    _ => panic!("TypeError at position {{}}: Cannot divide {} by {}", type_of(self), type_of(other)),
                }
            },
            Value::Long(val1) => {
                match other {
                    Value::Int(val2) => {
                        if val2.is_zero() { panic!("DivisionByZeroError at position {{}}: Cannot divide by zero"); }
                        if let Some(val1) = val1.to_i64() { return Value::Int(val1).divide(other); }
                        if let Some(val1) = long_to_f64(val1) { return Value::Float(val1 / *val2 as f64); }
                        let new = BigDecimal::from(val1.clone()) / BigDecimal::from(*val2);
                        if let Some(new) = new.to_f64() { return Value::Float(new); }
                        return Value::Decimal(new);
                    },
                    Value::Long(val2) => {
                        if val2.is_zero() { panic!("DivisionByZeroError at position {{}}: Cannot divide by zero"); }
                        if let Some(val1) = val1.to_i64() { return Value::Int(val1).divide(other); }
                        if let Some(val2) = val2.to_i64() { return self.divide(&Value::Int(val2)); }
                        if let Some(val1) = long_to_f64(val1) { return Value::Float(val1).divide(other); }
                        let new = BigDecimal::from(val1.clone()) / BigDecimal::from(val2.clone());
                        if let Some(new) = new.to_f64() { return Value::Float(new); }
                        return Value::Decimal(new);
                    },
                    Value::Float(val2) => {
                        if val2.is_zero() { panic!("DivisionByZeroError at position {{}}: Cannot divide by zero"); }
                        if let Some(val1) = long_to_f64(val1) { return Value::Float(val1 / val2); }
                        if let Ok(val2) = BigDecimal::try_from(*val2) {
                            let new = BigDecimal::from(val1.clone()) / val2;
                            if let Some(new) = new.to_f64() { return Value::Float(new); }
                            return Value::Decimal(new);
                        }
                        return Value::Float(val1.to_f64().unwrap() / val2);
                    },
                    Value::Decimal(val2) => {
                        if val2.is_zero() { panic!("DivisionByZeroError at position {{}}: Cannot divide by zero"); }
                        return Value::Decimal(BigDecimal::from(val1.clone()) / val2);
                    },
                    _ => panic!("TypeError at position {{}}: Cannot divide {} by {}", type_of(self), type_of(other)),
                }
            },
            Value::Float(val1) => {
                match other {
                    Value::Int(val2) => {
                        if val2.is_zero() { panic!("DivisionByZeroError at position {{}}: Cannot divide by zero"); }
                        return Value::Float(val1 / (*val2 as f64));
                    },
                    Value::Long(val2) => {
                        if val2.is_zero() { panic!("DivisionByZeroError at position {{}}: Cannot divide by zero"); }
                        if let Some(val2) = long_to_f64(val2) { return Value::Float(val1 / val2); }
                        if let Ok(val1) = BigDecimal::try_from(*val1) {
                            let new = val1 / BigDecimal::from(val2.clone());
                            if let Some(new) = new.to_f64() { return Value::Float(new); }
                            return Value::Decimal(new);
                        }
                        return Value::Float(val1 / val2.to_f64().unwrap());
                    },
                    Value::Float(val2) => {
                        if val2.is_zero() { panic!("DivisionByZeroError at position {{}}: Cannot divide by zero"); }
                        return Value::Float(val1 / val2);
                    },
                    Value::Decimal(val2) => {
                        if val2.is_zero() { panic!("DivisionByZeroError at position {{}}: Cannot divide by zero"); }
                        if let Ok(val1) = BigDecimal::try_from(*val1) { return Value::Decimal(val1 / val2) }
                        return Value::Float(val1 / val2.to_f64().unwrap());
                    },
                    _ => panic!("TypeError at position {{}}: Cannot divide {} by {}", type_of(self), type_of(other)),
                }
            },
            Value::Decimal(val1) => {
                match other {
                    Value::Int(val2) => {
                        if val2.is_zero() { panic!("DivisionByZeroError at position {{}}: Cannot divide by zero"); }
                        return Value::Decimal(val1 / BigDecimal::from(*val2));
                    },
                    Value::Long(val2) => {
                        if val2.is_zero() { panic!("DivisionByZeroError at position {{}}: Cannot divide by zero"); }
                        return Value::Decimal(val1 / BigDecimal::from(val2.clone()));
                    },
                    Value::Float(val2) => {
                        if val2.is_zero() { panic!("DivisionByZeroError at position {{}}: Cannot divide by zero"); }
                        if let Ok(val2) = BigDecimal::try_from(*val2) { return Value::Decimal(val1 / val2) }
                        return Value::Float(val1.to_f64().unwrap() / val2);
                    },
                    Value::Decimal(val2) => {
                        if val2.is_zero() { panic!("DivisionByZeroError at position {{}}: Cannot divide by zero"); }
                        return Value::Decimal(val1 / val2);
                    },
                    _ => panic!("TypeError at position {{}}: Cannot divide {} by {}", type_of(self), type_of(other)),
                }
            },
            _ => panic!("TypeError at position {{}}: Cannot divide {} by {}", type_of(self), type_of(other)),
        }
    }

    fn power(&self, other: &Self) -> Self {
        match self {
            Value::Int(val1) => {
                match other {
                    Value::Int(val2) => {
                        if val2 < &0 { return Value::Float(*val1 as f64).power(other); }
                        if val2 > &(u32::MAX as i64) { return self.power(&Value::Long(BigInt::from(*val2))); }
                        let new = val1.checked_pow(*val2 as u32);
                        if let Some(new) = new { return Value::Int(new); }
                        return Value::Long(BigInt::from(*val1).pow(*val2 as u32));
                    },
                    Value::Long(val2) => {
                        if val2 < &BigInt::zero() { panic!("TypeError at position {{}}: Cannot raise int by negative long"); }
                        return Value::Long(BigInt::from(*val1).pow(val2.to_biguint().unwrap()));
                    },
                    Value::Float(val2) => { return Value::Float((*val1 as f64).powf(*val2)); },
                    _ => panic!("TypeError at position {{}}: Cannot raise {} by {}", type_of(self), type_of(other)),
                }
            },
            Value::Long(val1) => {
                match other {
                    Value::Int(val2) => {
                        if let Some(val1) = val1.to_i64() { return Value::Int(val1).power(other); }
                        if val2 < &0 { panic!("TypeError at position {{}}: Cannot raise long by negative value"); }
                        if val2 > &(u32::MAX as i64) { return self.power(&Value::Long(BigInt::from(*val2))); }
                        return Value::Long(val1.pow(*val2 as u32));
                    },
                    Value::Long(val2) => {
                        if val2 < &BigInt::zero() { panic!("TypeError at position {{}}: Cannot raise int by negative long"); }
                        return Value::Long(val1.clone().pow(val2.to_biguint().unwrap()));
                    },
                    Value::Float(val2) => { return Value::Float(val1.to_f64().unwrap().powf(*val2)); },
                    _ => panic!("TypeError at position {{}}: Cannot raise {} by {}", type_of(self), type_of(other)),
                }
            },
            Value::Float(val1) => {
                match other {
                    Value::Int(val2) => { return Value::Float(val1.powf(*val2 as f64)); },
                    Value::Long(val2) => { return Value::Float(val1.powf(val2.to_f64().unwrap())); },
                    Value::Float(val2) => { return Value::Float(val1.pow(val2)); },
                    _ => panic!("TypeError at position {{}}: Cannot raise {} by {}", type_of(self), type_of(other)),
                }
            },
            _ => panic!("TypeError at position {{}}: Cannot raise {} by {}", type_of(self), type_of(other)),
        }
    }
}
