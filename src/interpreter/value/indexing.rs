use rust_decimal::prelude::ToPrimitive;
use std::rc::Rc;

use crate::error::{Result, Span};

use super::{types, Value, WrappedValue};

pub(crate) trait ToListIndex {
    fn to_list_index(&self, len: usize, span: &Span) -> Result<usize>;
}

impl<'tree> ToListIndex for Value<'tree> {
    fn to_list_index(&self, len: usize, span: &Span) -> Result<usize> {
        match self {
            Value::Number(num) => {
                if !num.fract().is_zero() {
                    error!(ValueError, *span, "List indices must be integers");
                }
                num.to_i128().unwrap().to_list_index(len, span)
            }
            _ => error!(
                TypeError,
                *span,
                "Type '{}' cannot be used for list indexing",
                types::type_of(self),
            ),
        }
    }
}

impl ToListIndex for i128 {
    fn to_list_index(&self, len: usize, span: &Span) -> Result<usize> {
        let len = len as i128;
        let idx = if self < &0 { len + self } else { *self };
        if idx >= len || idx < 0 {
            error!(
                ValueError,
                *span, "Index out of bounds: index is {self}, but length is {len}",
            );
        }
        match idx.to_usize() {
            Some(idx) => Ok(idx),
            None => error!(
                ValueError,
                *span,
                "Index too large: index is {idx}, but maximum list length is {}",
                usize::MAX,
            ),
        }
    }
}

impl<'tree> Value<'tree> {
    pub fn index(
        this: &WrappedValue<'tree>,
        index: &WrappedValue<'tree>,
        span: &Span,
    ) -> Result<WrappedValue<'tree>> {
        Ok(match (&*this.borrow(), &*index.borrow()) {
            (Value::List(list), Value::Number(_)) => {
                Rc::clone(&list[index.borrow().to_list_index(list.len(), span)?])
            }
            (Value::List(list), Value::Range { start, end }) => match (start, end) {
                (Some(start), Some(end)) => {
                    let start = start.to_list_index(list.len(), span)?;
                    let end = end.to_list_index(list.len(), span)?;
                    Value::List(list[start..=end].into()).wrapped()
                }
                (Some(start), None) => {
                    let start = start.to_list_index(list.len(), span)?;
                    Value::List(list[start..].into()).wrapped()
                }
                (None, Some(end)) => {
                    let end = end.to_list_index(list.len(), span)?;
                    Value::List(list[..=end].into()).wrapped()
                }
                (None, None) => Value::List(list[..].into()).wrapped(),
            },
            _ => error!(
                TypeError,
                *span,
                "Type '{}' cannot be indexed by type '{}'",
                types::type_of(&this.borrow()),
                types::type_of(&index.borrow()),
            ),
        })
    }
}
