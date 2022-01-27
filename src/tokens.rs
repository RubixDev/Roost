use std::fmt::Debug;
use crate::error::Location;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenType {
    Keyword,            // built-in keywords like 'true', 'var', or 'print'
    Identifier,         // variable and function names

    LParen,             // '('
    RParen,             // ')'
    LBrace,             // '{'
    RBrace,             // '}'

    String,             // string including quotes, token value does not include quotes
    Number,             // int or float
    RangeDots,          // '..' or '..='

    QuestionMark,       // '?' for ternary operator
    Colon,              // ':' for ternary operator
    Or,                 // '|'
    And,                // '&'
    Equal,              // '=='
    NotEqual,           // '!='
    LessThan,           // '<'
    GreaterThan,        // '>'
    LessThanOrEqual,    // '<='
    GreaterThanOrEqual, // '>='
    Plus,               // '+'
    Minus,              // '-'
    Multiply,           // '*'
    Power,              // '**'
    Divide,             // '/'
    Modulo,             // '%'
    IntDivide,          // '\'
    Not,                // '!'

    Assign,             // '='
    PlusAssign,         // '+='
    MinusAssign,        // '-='
    MultiplyAssign,     // '*='
    DivideAssign,       // '/='

    Comma,              // ','

    EOL,                // End Of Line: \n or ';'
    EOF,                // End Of File
}

#[derive(Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub location: Location,
}

impl Token {
    pub fn new(token_type: TokenType, value: &str, location: Location) -> Self {
        return Token { token_type, value: String::from(value), location };
    }

    pub fn matches(&self, token_type: TokenType, value: &str) -> bool {
        return self.token_type == token_type && &self.value == value;
    }

    pub fn dummy() -> Self {
        return Token {
            token_type: TokenType::EOF,
            value: String::from("EOF"),
            location: Location::new(String::new()),
        };
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "( {:?} | {:?} )", self.token_type, self.value)
    }
}
