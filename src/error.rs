use std::fmt::Debug;

pub type Result<T> = std::result::Result<T, Error>;

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

#[derive(Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
    pub start: Location,
    pub end: Location,
}

impl Error {
    pub fn new(kind: ErrorKind, message: String, start: Location, end: Location) -> Self {
        Self {
            kind,
            message,
            start,
            end,
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} at {:?}  {}", self.kind, self.start, self.message,)
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
    RuntimeError,
}
