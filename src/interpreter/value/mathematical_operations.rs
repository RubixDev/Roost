use super::{types, Value};
use crate::error::{Result, Span};
use rust_decimal::{prelude::ToPrimitive, MathematicalOps};

impl Value<'_> {
    pub fn add(&self, other: &Self, span: &Span) -> Result<Self> {
        Ok(match (self, other) {
            (Value::Number(left), Value::Number(right)) => Value::Number(
                match left.checked_add(*right) {
                    Some(res) => res,
                    None => error!(OverflowError, *span, "Addition resulted in overflow"),
                }
                .normalize(),
            ),
            (Value::String(left), right) => Value::String(left.to_string() + &right.to_string()),
            (left, Value::String(right)) => Value::String(left.to_string() + right),
            (Value::List(left), Value::List(right)) => {
                let mut new_list = left.clone();
                new_list.append(&mut right.clone());
                Value::List(new_list)
            }
            _ => error!(
                TypeError,
                *span,
                "Cannot add {} to {}",
                types::type_of(other),
                types::type_of(self),
            ),
        })
    }

    pub fn sub(&self, other: &Self, span: &Span) -> Result<Self> {
        Ok(match (self, other) {
            (Value::Number(left), Value::Number(right)) => Value::Number(
                match left.checked_sub(*right) {
                    Some(res) => res,
                    None => error!(OverflowError, *span, "Subtraction resulted in overflow"),
                }
                .normalize(),
            ),
            _ => error!(
                TypeError,
                *span,
                "Cannot subtract {} from {}",
                types::type_of(other),
                types::type_of(self),
            ),
        })
    }

    pub fn mul(&self, other: &Self, span: &Span) -> Result<Self> {
        Ok(match (self, other) {
            (Value::Number(left), Value::Number(right)) => Value::Number(
                match left.checked_mul(*right) {
                    Some(res) => res,
                    None => error!(OverflowError, *span, "Multiplication resulted in overflow"),
                }
                .normalize(),
            ),
            (Value::String(left), Value::Number(right))
            | (Value::Number(right), Value::String(left)) => {
                if !right.fract().is_zero() {
                    error!(
                        ValueError,
                        *span, "Cannot multiply string with fractional number",
                    );
                }
                let mut string = String::new();
                let mut i = right.to_i128().unwrap();
                while i > 0 {
                    string += left;
                    i -= 1;
                }
                Value::String(string)
            }
            _ => error!(
                TypeError,
                *span,
                "Cannot multiply {} with {}",
                types::type_of(self),
                types::type_of(other),
            ),
        })
    }

    pub fn div(&self, other: &Self, span: &Span) -> Result<Self> {
        Ok(match (self, other) {
            (Value::Number(left), Value::Number(right)) => {
                if right.is_zero() {
                    error!(DivisionByZeroError, *span, "Cannot divide by zero")
                }
                Value::Number(
                    match left.checked_div(*right) {
                        Some(res) => res,
                        None => {
                            error!(OverflowError, *span, "Division resulted in overflow")
                        }
                    }
                    .normalize(),
                )
            }
            _ => error!(
                TypeError,
                *span,
                "Cannot divide {} by {}",
                types::type_of(self),
                types::type_of(other),
            ),
        })
    }

    pub fn pow(&self, other: &Self, span: &Span) -> Result<Self> {
        Ok(match (self, other) {
            (Value::Number(left), Value::Number(right)) => Value::Number(
                match left.checked_powd(*right) {
                    Some(res) => res,
                    None => error!(OverflowError, *span, "Power resulted in overflow"),
                }
                .normalize(),
            ),
            _ => error!(
                TypeError,
                *span,
                "Cannot raise {} by {}",
                types::type_of(self),
                types::type_of(other),
            ),
        })
    }

    pub fn rem(&self, other: &Self, span: &Span) -> Result<Self> {
        Ok(match (self, other) {
            (Value::Number(left), Value::Number(right)) => {
                if right.is_zero() {
                    error!(DivisionByZeroError, *span, "Cannot divide by zero")
                }
                Value::Number(
                    match left.checked_rem(*right) {
                        Some(res) => res,
                        None => {
                            error!(
                                OverflowError,
                                *span, "Remainder division resulted in overflow",
                            )
                        }
                    }
                    .normalize(),
                )
            }
            _ => error!(
                TypeError,
                *span,
                "Cannot divide {} by {}",
                types::type_of(self),
                types::type_of(other),
            ),
        })
    }

    pub fn div_floor(&self, other: &Self, span: &Span) -> Result<Self> {
        Ok(match (self, other) {
            (Value::Number(left), Value::Number(right)) => {
                if right.is_zero() {
                    error!(DivisionByZeroError, *span, "Cannot divide by zero")
                }
                Value::Number(
                    match left.checked_div(*right) {
                        Some(res) => res,
                        None => {
                            error!(OverflowError, *span, "Division resulted in overflow")
                        }
                    }
                    .normalize()
                    .floor(),
                )
            }
            _ => error!(
                TypeError,
                *span,
                "Cannot divide {} by {}",
                types::type_of(self),
                types::type_of(other),
            ),
        })
    }
}
