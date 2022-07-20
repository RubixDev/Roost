use super::{types, Value};
use crate::error::{Location, Result};
use rust_decimal::{prelude::ToPrimitive, MathematicalOps};

impl Value<'_> {
    pub fn add(&self, other: &Self, start: &Location, end: &Location) -> Result<Self> {
        Ok(match (self, other) {
            (Value::Number(left), Value::Number(right)) => Value::Number(
                match left.checked_add(*right) {
                    Some(res) => res,
                    None => error!(OverflowError, *start, *end, "Addition resulted in overflow"),
                }
                .normalize(),
            ),
            (Value::Number(left), Value::String(right)) => Value::String(left.to_string() + right),
            (Value::String(left), right) | (right, Value::String(left)) => {
                Value::String(left.to_string() + &right.to_string())
            }
            _ => error!(
                TypeError,
                *start,
                *end,
                "Cannot add {} to {}",
                types::type_of(self),
                types::type_of(other)
            ),
        })
    }

    pub fn sub(&self, other: &Self, start: &Location, end: &Location) -> Result<Self> {
        Ok(match (self, other) {
            (Value::Number(left), Value::Number(right)) => Value::Number(
                match left.checked_sub(*right) {
                    Some(res) => res,
                    None => error!(
                        OverflowError,
                        *start, *end, "Subtraction resulted in overflow"
                    ),
                }
                .normalize(),
            ),
            _ => error!(
                TypeError,
                *start,
                *end,
                "Cannot subtract {} from {}",
                types::type_of(other),
                types::type_of(self)
            ),
        })
    }

    pub fn mul(&self, other: &Self, start: &Location, end: &Location) -> Result<Self> {
        Ok(match (self, other) {
            (Value::Number(left), Value::Number(right)) => Value::Number(
                match left.checked_mul(*right) {
                    Some(res) => res,
                    None => error!(
                        OverflowError,
                        *start, *end, "Multiplication resulted in overflow"
                    ),
                }
                .normalize(),
            ),
            (Value::String(left), Value::Number(right))
            | (Value::Number(right), Value::String(left)) => {
                if !right.fract().is_zero() {
                    error!(
                        ValueError,
                        *start, *end, "Cannot multiply string with fractional number"
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
                *start,
                *end,
                "Cannot multiply {} with {}",
                types::type_of(self),
                types::type_of(other)
            ),
        })
    }

    pub fn div(&self, other: &Self, start: &Location, end: &Location) -> Result<Self> {
        Ok(match (self, other) {
            (Value::Number(left), Value::Number(right)) => {
                if right.is_zero() {
                    error!(DivisionByZeroError, *start, *end, "Cannot divide by zero")
                }
                Value::Number(
                    match left.checked_div(*right) {
                        Some(res) => res,
                        None => {
                            error!(OverflowError, *start, *end, "Division resulted in overflow")
                        }
                    }
                    .normalize(),
                )
            }
            _ => error!(
                TypeError,
                *start,
                *end,
                "Cannot divide {} by {}",
                types::type_of(self),
                types::type_of(other)
            ),
        })
    }

    pub fn pow(&self, other: &Self, start: &Location, end: &Location) -> Result<Self> {
        Ok(match (self, other) {
            (Value::Number(left), Value::Number(right)) => Value::Number(
                match left.checked_powd(*right) {
                    Some(res) => res,
                    None => error!(OverflowError, *start, *end, "Power resulted in overflow"),
                }
                .normalize(),
            ),
            _ => error!(
                TypeError,
                *start,
                *end,
                "Cannot raise {} by {}",
                types::type_of(self),
                types::type_of(other)
            ),
        })
    }

    pub fn rem(&self, other: &Self, start: &Location, end: &Location) -> Result<Self> {
        Ok(match (self, other) {
            (Value::Number(left), Value::Number(right)) => {
                if right.is_zero() {
                    error!(DivisionByZeroError, *start, *end, "Cannot divide by zero")
                }
                Value::Number(
                    match left.checked_rem(*right) {
                        Some(res) => res,
                        None => {
                            error!(
                                OverflowError,
                                *start, *end, "Remainder division resulted in overflow"
                            )
                        }
                    }
                    .normalize(),
                )
            }
            _ => error!(
                TypeError,
                *start,
                *end,
                "Cannot divide {} by {}",
                types::type_of(self),
                types::type_of(other)
            ),
        })
    }

    pub fn div_floor(&self, other: &Self, start: &Location, end: &Location) -> Result<Self> {
        Ok(match (self, other) {
            (Value::Number(left), Value::Number(right)) => {
                if right.is_zero() {
                    error!(DivisionByZeroError, *start, *end, "Cannot divide by zero")
                }
                Value::Number(
                    match left.checked_div(*right) {
                        Some(res) => res,
                        None => {
                            error!(OverflowError, *start, *end, "Division resulted in overflow")
                        }
                    }
                    .normalize()
                    .floor(),
                )
            }
            _ => error!(
                TypeError,
                *start,
                *end,
                "Cannot divide {} by {}",
                types::type_of(self),
                types::type_of(other)
            ),
        })
    }
}
