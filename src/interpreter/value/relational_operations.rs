use super::{types, Value};
use crate::error::{Result, Span};

macro_rules! rel_op {
    ($name:ident, $op:tt) => {
        pub fn $name(&self, other: &Self, span: Span) -> Result<Self> {
            Ok(match (self, other) {
                (Value::Number(left), Value::Number(right)) => Value::Bool(left $op right),
                _ => error!(
                    TypeError,
                    span,
                    "Cannot compare {} with {}",
                    types::type_of(self),
                    types::type_of(other)
                ),
            })
        }
    };
}

impl Value<'_> {
    rel_op!(lt, <);
    rel_op!(le, <=);
    rel_op!(gt, >);
    rel_op!(ge, >=);
}
