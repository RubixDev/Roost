use std::fmt::{Display, Debug};

pub type Result<'f, T> = std::result::Result<T, Error<'f>>;

macro_rules! error_val {
    ($kind:ident, $start:expr, $end:expr, $($arg:tt)*) => {
        $crate::error::Error::new(
            $crate::error::ErrorKind::$kind,
            format!($($arg)*),
            $start,
            $end,
        )
    };
}

macro_rules! error {
    ($kind:ident, $start:expr, $end:expr, $($arg:tt)*) => {
        return Err(error_val!($kind, $start, $end, $($arg)*))
    };
}

#[derive(Clone, Copy, PartialEq)]
pub struct Location<'f> {
    pub filename: &'f str,
    pub line: usize,
    pub column: usize,
    pub index: usize,
}

impl<'f> Location<'f> {
    pub fn new(filename: &'f str) -> Self {
        Self {
            filename,
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

impl<'f> Debug for Location<'f> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.filename, self.line, self.column)
    }
}

#[derive(Debug, Clone)]
pub struct Error<'f> {
    pub kind: ErrorKind,
    pub message: String,
    pub start: Location<'f>,
    pub end: Location<'f>,
}

impl<'f> Error<'f> {
    pub fn new(kind: ErrorKind, message: String, start: Location<'f>, end: Location<'f>) -> Self {
        Self {
            kind,
            message,
            start,
            end,
        }
    }
}

impl Display for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} at {}:{}:{}  {}",
            self.kind, self.start.filename, self.start.line, self.start.column, self.message,
        )
    }
}

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
}
