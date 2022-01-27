use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

macro_rules! error {
    ($kind:ident, $pos:expr, $($arg:tt)*) => {
        return Err(
            crate::error::Error::new(
                crate::error::ErrorKind::$kind,
                format!($($arg)*),
                $pos,
            )
        )
    };
}

#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    pub filename: String,
    pub line: usize,
    pub column: usize,
    index: usize,
}

impl Location {
    pub fn new(filename: String) -> Self {
        return Location {
            filename,
            line: 1,
            column: 1,
            index: 0,
        };
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

pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
    pub location: Location,
}

impl Error {
    pub fn new(kind: ErrorKind, message: String, location: Location) -> Self {
        return Error { kind, message, location };
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} at {}:{}:{}  {}",
            self.kind,
            self.location.filename,
            self.location.line,
            self.location.column,
            self.message,
        )
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    SyntaxError,
    TypeError,
    ReferenceError,
    ValueError,
    DivisionByZeroError,
    OverflowError,
}
