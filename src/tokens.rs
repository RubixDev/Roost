use crate::error::Location;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Clone, PartialEq)]
pub struct Token<'f> {
    pub token_type: TokenType,
    pub value: String,
    pub start: Location<'f>,
    pub end: Location<'f>,
}

impl<'f> Token<'f> {
    pub fn new(
        token_type: TokenType,
        value: String,
        start: Location<'f>,
        end: Location<'f>,
    ) -> Self {
        Token {
            token_type,
            value,
            start,
            end,
        }
    }

    pub fn dummy() -> Self {
        return Token {
            token_type: TokenType::Unknown,
            value: String::from("Unknown"),
            start: Location::new(""),
            end: Location::new(""),
        };
    }
}

impl Debug for Token<'_> {
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
