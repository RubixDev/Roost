use std::{collections::HashMap, fmt::Debug};

use crate::interpreter::value::{ToValue, Value};

pub type Result<T> = std::result::Result<T, Error>;

macro_rules! error_val {
    ($kind:ident, ($start:expr, $end:expr), $($arg:tt)*) => {
        error_val!($kind, $crate::error::Span::new($start, $end), $($arg)*)
    };
    ($kind:ident, $span:expr, $($arg:tt)*) => {
        $crate::error::Error::new(
            $crate::error::ErrorKind::$kind,
            format!($($arg)*),
            $span,
        )
    };
}

macro_rules! error {
    ($($arg:tt)*) => {
        return Err(error_val!($($arg)*))
    };
}

/////////////////////////////////////////////

#[derive(Clone, Copy, PartialEq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub index: usize,
}

impl Location {
    pub fn new() -> Self {
        Self {
            line: 1,
            column: 1,
            index: 0,
        }
    }

    pub fn advance(&mut self, next_line: bool) {
        self.index += 1;
        if next_line {
            self.column = 1;
            self.line += 1;
        } else {
            self.column += 1;
        }
    }
}

impl Default for Location {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl ToValue for Location {
    fn to_value<'tree>(&self) -> Value<'tree> {
        Value::Object(HashMap::from([
            ("line", Value::Number(self.line.into()).wrapped()),
            ("column", Value::Number(self.column.into()).wrapped()),
            ("index", Value::Number(self.index.into()).wrapped()),
        ]))
    }
}

/////////////////////////////////////////////

#[derive(Clone, Copy, PartialEq, Default)]
pub struct Span {
    pub start: Location,
    pub end: Location,
}

impl Span {
    pub fn new(start: Location, end: Location) -> Self {
        Self { start, end }
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}..{:?}", self.start, self.end)
    }
}

impl ToValue for Span {
    fn to_value<'tree>(&self) -> Value<'tree> {
        Value::Object(HashMap::from([
            ("start", self.start.to_value().wrapped()),
            ("end", self.end.to_value().wrapped()),
        ]))
    }
}

/////////////////////////////////////////////

#[derive(Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Span,
}

impl Error {
    pub fn new(kind: ErrorKind, message: String, span: Span) -> Self {
        Self {
            kind,
            message,
            span,
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} at {:?}  {}", self.kind, self.span, self.message,)
    }
}

impl ToValue for Error {
    fn to_value<'tree>(&self) -> Value<'tree> {
        Value::Object(HashMap::from([
            ("kind", Value::String(format!("{:?}", self.kind)).wrapped()),
            ("message", Value::String(self.message.clone()).wrapped()),
            ("span", self.span.to_value().wrapped()),
        ]))
    }
}

/////////////////////////////////////////////

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum ErrorKind {
    SyntaxError,
    TypeError,
    ReferenceError,
    ValueError,
    DivisionByZeroError,
    OverflowError,
    SystemError,
    RuntimeError,
}
