use rust_decimal::{prelude::ToPrimitive, Decimal};

use crate::error::{Result, Span};

use super::Value;

macro_rules! bitwise_op {
    ($name:ident, $op:tt) => {
        pub fn $name(&self, other: &Self, span: Span) -> Result<Self> {
            Ok(match (self, other) {
                (Value::Number(left), Value::Number(right))
                    if left.fract().is_zero() && right.fract().is_zero() =>
                {
                    Value::Number(Decimal::from(
                        left.to_i128().unwrap() $op right.to_i128().unwrap(),
                    ))
                }
                (Value::Number(left), Value::Bool(right))
                | (Value::Bool(right), Value::Number(left))
                    if left.fract().is_zero() =>
                {
                    Value::Number(Decimal::from(left.to_i128().unwrap() $op *right as i128))
                }
                (Value::Bool(left), Value::Bool(right)) => bitwise_op!(@both_bools left, right, $op),
                _ => error!(
                    ValueError,
                    span,
                    "Bitwise operations require integers or booleans on both sides",
                ),
            })
        }
    };
    (@both_bools $left:ident, $right:ident, <<) => {
        Value::Number(Decimal::from((*$left as i8) << *$right as i8))
    };
    (@both_bools $left:ident, $right:ident, >>) => {
        Value::Number(Decimal::from((*$left as i8) >> *$right as i8))
    };
    (@both_bools $left:ident, $right:ident, $op:tt) => {
        Value::Bool($left $op $right)
    };
}

impl Value<'_> {
    bitwise_op!(shl, <<);
    bitwise_op!(shr, >>);
    bitwise_op!(or, |);
    bitwise_op!(xor, ^);
    bitwise_op!(and, &);
}
