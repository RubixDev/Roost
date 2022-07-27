use std::{marker::PhantomData, rc::Rc, slice::Iter, str::Chars};

use crate::error::{Result, Span};

use super::{types, Value, WrappedValue};

impl<'tree> Value<'tree> {
    pub fn to_iter(
        &self,
        span: &Span,
    ) -> Result<Box<dyn Iterator<Item = WrappedValue<'tree>> + '_>> {
        match self {
            Value::String(val) => Ok(Box::new(StringIterator::new(val))),
            Value::Range { start, end } => match (start, end) {
                (Some(start), Some(end)) => Ok(Box::new(RangeIterator::new(*start..=*end))),
                (Some(start), None) => Ok(Box::new(RangeIterator::new(*start..))),
                _ => error!(
                    ValueError,
                    *span, "Cannot iterate over ranges with open start",
                ),
            },
            Value::List(list) => Ok(Box::new(ListIterator::new(list))),
            _ => error!(
                TypeError,
                *span,
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
    type Item = WrappedValue<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|char| Value::String(char.to_string()).wrapped())
    }
}

struct RangeIterator<'tree, Range>
where
    Range: Iterator<Item = i128>,
{
    inner: Range,
    _tree: PhantomData<&'tree ()>,
}

impl<'tree, Range> RangeIterator<'tree, Range>
where
    Range: Iterator<Item = i128>,
{
    fn new(range: Range) -> Self {
        Self {
            inner: range,
            _tree: PhantomData,
        }
    }
}

impl<'tree, Range> Iterator for RangeIterator<'tree, Range>
where
    Range: Iterator<Item = i128>,
{
    type Item = WrappedValue<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|num| Value::Number(num.into()).wrapped())
    }
}

struct ListIterator<'src, 'tree> {
    inner: Iter<'src, WrappedValue<'tree>>,
    _tree: PhantomData<&'tree ()>,
}

impl<'src, 'tree> ListIterator<'src, 'tree> {
    fn new(list: &'src [WrappedValue<'tree>]) -> Self {
        Self {
            inner: list.iter(),
            _tree: PhantomData,
        }
    }
}

impl<'tree> Iterator for ListIterator<'_, 'tree> {
    type Item = WrappedValue<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Rc::clone)
    }
}
