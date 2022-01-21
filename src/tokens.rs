use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenType {
    Keyword,        // built-in keywords like 'true', 'var', or 'print'
    Identifier,     // variable and function names

    LParen,         // '('
    RParen,         // ')'
    LBrace,         // '{'
    RBrace,         // '}'

    String,         // string including quotes, token value does not include quotes
    Number,         // int or float
    RangeDots,      // '..' or '..='

    QuestionMark,   // '?' for ternary operator
    Colon,          // ':' for ternary operator
    Or,             // '|'
    And,            // '&'
    EQ,             // '=='
    NEQ,            // '!='
    LT,             // '<'
    GT,             // '>'
    LTE,            // '<='
    GTE,            // '>='
    Plus,           // '+'
    Minus,          // '-'
    Multiply,       // '*'
    Divide,         // '/'
    Not,            // '!'

    Assign,         // '='
    PlusAssign,     // '+='
    MinusAssign,    // '-='
    MultiplyAssign, // '*='
    DivideAssign,   // '/='

    Comma,          // ','
    Dot,            // '.'

    EOL,            // End Of Line: \n or ';'
    EOF,            // End Of File
}

pub struct Token {
    pub token_type: TokenType,
    pub value: String,
}

impl Token {
    pub fn new(token_type: TokenType, value: &str) -> Token {
        return Token { token_type, value: String::from(value) };
    }

    pub fn is_type(&self, token_type: TokenType) -> bool {
        return self.token_type == token_type;
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "( {:?} | {:?} )", self.token_type, self.value)
    }
}
