use crate::error::Location;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    Identifier, // name for variable, function or class

    LParen, // '('
    RParen, // ')'
    LBrace, // '{'
    RBrace, // '}'

    String,        // string including quotes, token value does not include quotes
    Number,        // int or float
    Dots,          // '..'
    DotsInclusive, // '..='

    Or,                 // '||'
    And,                // '&&'
    BitOr,              // '|'
    BitXor,             // '^'
    BitAnd,             // '&'
    Equal,              // '=='
    NotEqual,           // '!='
    LessThan,           // '<'
    GreaterThan,        // '>'
    LessThanOrEqual,    // '<='
    GreaterThanOrEqual, // '>='
    ShiftRight,         // '>>'
    ShiftLeft,          // '<<'
    Plus,               // '+'
    Minus,              // '-'
    Multiply,           // '*'
    Power,              // '**'
    Divide,             // '/'
    Modulo,             // '%'
    IntDivide,          // '\'
    Not,                // '!'

    Assign,           // '='
    PlusAssign,       // '+='
    MinusAssign,      // '-='
    MultiplyAssign,   // '*='
    DivideAssign,     // '/='
    ModuloAssign,     // '%='
    IntDivideAssign,  // '\='
    PowerAssign,      // '**='
    ShiftLeftAssign,  // '<<='
    ShiftRightAssign, // '>>='
    BitOrAssign,      // '|='
    BitAndAssign,     // '&='
    BitXorAssign,     // '^='

    Comma, // ','
    Dot,   // '.'

    // Keywords
    Var,
    True,
    False,
    Null,
    If,
    Else,
    Fun,
    Static,
    Loop,
    While,
    For,
    Class,
    In,
    Return,
    Break,
    Continue,
    Try,
    Catch,

    Unknown,
    Semicolon,
    Eof,
}

#[derive(Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: Option<String>,
    pub start: Location,
    pub end: Location,
}

impl Token {
    pub fn new(token_type: TokenType, value: String, start: Location, end: Location) -> Self {
        Token {
            token_type,
            value: Some(value),
            start,
            end,
        }
    }

    pub fn dummy() -> Self {
        Token {
            token_type: TokenType::Unknown,
            value: Some("Unknown".to_string()),
            start: Location::new(),
            end: Location::new(),
        }
    }

    pub fn value(&self) -> &str {
        match &self.value {
            Some(value) => value,
            None => "",
        }
    }

    pub fn take_value(&mut self) -> String {
        self.value.take().unwrap_or_default()
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "( {:?} | {:?} | {}:{}..{}:{} )",
            self.token_type,
            self.value,
            self.start.line,
            self.start.column,
            self.end.line,
            self.end.column
        )
    }
}
