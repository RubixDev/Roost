use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

macro_rules! error {
    ($kind:ident, $($arg:tt)*) => {
        return Err(crate::error::Error::new(crate::error::ErrorKind::$kind, format!($($arg)*)))
    };
}

pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, message: String) -> Self {
        return Error { kind, message };
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    SyntaxError,
    TypeError,
    ReferenceError,
    ValueError,
    DivisionByZeroError,
}
