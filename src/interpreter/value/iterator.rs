use std::{marker::PhantomData, ops::RangeInclusive, str::Chars};

use crate::error::{Result, Span};

use super::{types, Value};

impl<'tree> Value<'tree> {
    pub fn to_iter(&self, span: Span) -> Result<Box<dyn Iterator<Item = Value<'tree>> + '_>> {
        match self {
            Value::String(val) => Ok(Box::new(StringIterator::new(val))),
            Value::Range { start, end } => Ok(Box::new(RangeIterator::new(*start..=*end))),
            _ => error!(
                TypeError,
                span,
                "Cannot iterate over type '{}'",
                types::type_of(self)
            ),
        }
    }
}

struct StringIterator<'src, 'tree> {
    inner: Chars<'src>,
    _tree: PhantomData<&'tree ()>,
}

impl<'src, 'tree> StringIterator<'src, 'tree> {
    fn new(string: &'src str) -> Self {
        Self {
            inner: string.chars(),
            _tree: PhantomData,
        }
    }
}

impl<'src, 'tree> Iterator for StringIterator<'src, 'tree> {
    type Item = Value<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|char| Value::String(char.to_string()))
    }
}

struct RangeIterator<'tree> {
    inner: RangeInclusive<i128>,
    _tree: PhantomData<&'tree ()>,
}

impl<'tree> RangeIterator<'tree> {
    fn new(range: RangeInclusive<i128>) -> Self {
        Self {
            inner: range,
            _tree: PhantomData,
        }
    }
}

impl<'tree> Iterator for RangeIterator<'tree> {
    type Item = Value<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|num| Value::Number(num.into()))
    }
}
