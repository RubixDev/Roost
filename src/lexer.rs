use crate::{
    error::{Error, Location},
    tokens::{Token, TokenType},
};
use std::str::{self, Chars};

const DIGITS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
const OCTAL_DIGITS: [char; 8] = ['0', '1', '2', '3', '4', '5', '6', '7'];
const HEX_DIGITS: [char; 22] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'a', 'B', 'b', 'C', 'c', 'D', 'd', 'E',
    'e', 'F', 'f',
];
const LETTERS: [char; 53] = [
    'A', 'a', 'B', 'b', 'C', 'c', 'D', 'd', 'E', 'e', 'F', 'f', 'G', 'g', 'H', 'h', 'I', 'i', 'J',
    'j', 'K', 'k', 'L', 'l', 'M', 'm', 'N', 'n', 'O', 'o', 'P', 'p', 'Q', 'q', 'R', 'r', 'S', 's',
    'T', 't', 'U', 'u', 'V', 'v', 'W', 'w', 'X', 'x', 'Y', 'y', 'Z', 'z', '_',
];

const ESCAPE_CHAR: [char; 10] = ['\\', '\'', '"', 'a', 'b', 'f', 'n', 'r', 't', 'v'];

macro_rules! lex_error {
    ($self:ident, $start:ident, $($arg:tt)*) => {
        return Err((
            error_val!(SyntaxError, $start, $self.location, $($arg)*),
            Token::new(
                TokenType::Unknown,
                "Unknown".to_string(),
                $start,
                $self.location,
            ),
        ))
    };
}

macro_rules! char_construct {
    ($self:ident, $type_single:ident, $type_with_eq:tt, $type_double:tt, $type_double_with_eq:tt $(,)?) => {
        return Ok($self.make_char_construct(
            TokenType::$type_single,
            char_construct!(@optional $type_with_eq),
            char_construct!(@optional $type_double),
            char_construct!(@optional $type_double_with_eq),
        ))
    };
    (@optional _) => { None };
    (@optional $type:ident) => { Some(TokenType::$type) };
}

type LexResult<'f, T> = std::result::Result<T, (Error<'f>, Token<'f>)>;

#[derive(Debug, Clone)]
pub struct Lexer<'i, 'f> {
    input: Chars<'i>,
    current_char: Option<char>,
    location: Location<'f>,
}

impl<'i, 'f> Lexer<'i, 'f> {
    pub fn new(input: &'i str, filename: &'f str) -> Self {
        let mut lexer = Lexer {
            input: input.chars(),
            current_char: None,
            location: Location::new(filename),
        };
        lexer.advance();
        lexer
    }

    pub fn next_token(&mut self) -> LexResult<'f, Token> {
        while let Some(current_char) = self.current_char {
            match current_char {
                ' ' | '\t' | '\r' | '\n' => self.advance(),
                '"' | '\'' => return self.make_string(),
                '.' => return self.make_dot(),
                '/' => {
                    if let Some(token) = self.make_slash() {
                        return Ok(token);
                    }
                }
                '*' => char_construct!(self, Multiply, MultiplyAssign, Power, PowerAssign),
                '(' => char_construct!(self, LParen, _, _, _),
                ')' => char_construct!(self, RParen, _, _, _),
                '{' => char_construct!(self, LBrace, _, _, _),
                '}' => char_construct!(self, RBrace, _, _, _),
                ',' => char_construct!(self, Comma, _, _, _),
                ';' => char_construct!(self, Semicolon, _, _, _),
                '|' => char_construct!(self, BitOr, BitOrAssign, Or, _),
                '&' => char_construct!(self, BitAnd, BitAndAssign, And, _),
                '=' => char_construct!(self, Assign, Equal, _, _),
                '!' => char_construct!(self, Not, NotEqual, _, _),
                '<' => char_construct!(self, LessThan, LessThanOrEqual, ShiftLeft, ShiftLeftAssign),
                '>' => char_construct!(
                    self,
                    GreaterThan,
                    GreaterThanOrEqual,
                    ShiftRight,
                    ShiftRightAssign,
                ),
                '+' => char_construct!(self, Plus, PlusAssign, _, _),
                '-' => char_construct!(self, Minus, MinusAssign, _, _),
                '%' => char_construct!(self, Modulo, ModuloAssign, _, _),
                '\\' => char_construct!(self, IntDivide, IntDivideAssign, _, _),
                _ => {
                    if DIGITS.contains(&current_char) {
                        return Ok(self.make_number());
                    } else if LETTERS.contains(&current_char) {
                        return Ok(self.make_name());
                    } else {
                        let start_pos = self.location;
                        self.advance();
                        lex_error!(self, start_pos, "Illegal character '{}'", current_char);
                    }
                }
            }
        }

        let start_pos = self.location;
        self.location.advance(false);
        return Ok(Token::new(
            TokenType::Eof,
            "EOF".to_string(),
            start_pos,
            self.location,
        ));
    }

    fn advance(&mut self) {
        if let Some(current_char) = self.current_char {
            self.location.advance(current_char == '\n')
        }
        self.current_char = self.input.next();
    }

    // TODO: try to remove clone here
    fn next_char(&self) -> Option<char> {
        self.input.clone().next()
    }

    // ----------------------------------------

    fn make_char_construct(
        &mut self,
        type_single: TokenType,
        type_with_eq: Option<TokenType>,
        type_double: Option<TokenType>,
        type_double_with_eq: Option<TokenType>,
    ) -> Token<'f> {
        let start_location = self.location;
        let char = self.current_char.unwrap();
        self.advance();
        match (
            type_with_eq,
            &type_double,
            &type_double_with_eq,
            self.current_char,
        ) {
            (Some(ty), .., Some('=')) => {
                self.advance();
                Token::new(ty, format!("{char}="), start_location, self.location)
            }
            (_, Some(_), _, Some(c)) | (_, _, Some(_), Some(c)) if c == char => {
                self.advance();
                match (type_double, type_double_with_eq, self.current_char) {
                    (_, Some(ty), Some('=')) => {
                        self.advance();
                        Token::new(ty, format!("{char}{char}="), start_location, self.location)
                    }
                    (Some(ty), ..) => {
                        Token::new(ty, format!("{char}{char}"), start_location, self.location)
                    }
                    // can panic when all this is true:
                    // - `type_double` is `None`
                    // - `type_double_with_eq` is `Some(_)`
                    // - `self.current_char` is not `Some('=')`
                    // but we never call this function that way
                    _ => unreachable!(),
                }
            }
            _ => Token::new(type_single, char.to_string(), start_location, self.location),
        }
    }

    fn make_string(&mut self) -> LexResult<'f, Token> {
        let start_location = self.location;
        let start_quote = self.current_char;
        let mut string = String::new();

        self.advance(); // start quote
        while ![start_quote, Some('\\'), None].contains(&self.current_char) {
            string.push(self.current_char.unwrap());
            self.advance();
        }
        while self.current_char == Some('\\') {
            let escape_pos = self.location;
            self.advance(); // backslash
            if self.current_char == None {
                lex_error!(self, escape_pos, "Invalid escape sequence")
            }
            let current_char = self.current_char.unwrap();

            if ESCAPE_CHAR.contains(&current_char) {
                string.push(match current_char {
                    '\\' => '\\',
                    '\'' => '\'',
                    '"' => '"',
                    'a' => '\x07',
                    'b' => '\x08',
                    'f' => '\x0c',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    'v' => '\x0b',
                    _ => unreachable!(),
                });
                self.advance();
            } else if OCTAL_DIGITS.contains(&current_char) {
                string.push(self.escape_sequence(&current_char, escape_pos, false, 2)?);
            } else if current_char == 'x' {
                string.push(self.escape_sequence(&current_char, escape_pos, true, 2)?);
            } else if current_char == 'u' {
                string.push(self.escape_sequence(&current_char, escape_pos, true, 4)?);
            } else if current_char == 'U' {
                string.push(self.escape_sequence(&current_char, escape_pos, true, 8)?);
            } else {
                lex_error!(self, escape_pos, "Invalid escape sequence");
            }

            while ![start_quote, Some('\\'), None].contains(&self.current_char) {
                string.push(self.current_char.unwrap());
                self.advance();
            }
        }
        self.advance(); // end quote

        return Ok(Token::new(
            TokenType::String,
            string,
            start_location,
            self.location,
        ));
    }

    fn escape_sequence(
        &mut self,
        current_char: &char,
        start_pos: Location<'f>,
        is_hex: bool,
        digits: u8,
    ) -> LexResult<'f, char> {
        let mut esc = if is_hex {
            String::new()
        } else {
            current_char.to_string()
        };
        self.advance();
        for _ in 0..digits {
            if self.current_char == None
                || if is_hex {
                    !HEX_DIGITS.contains(&self.current_char.unwrap())
                } else {
                    !OCTAL_DIGITS.contains(&self.current_char.unwrap())
                }
            {
                lex_error!(self, start_pos, "Invalid escape sequence");
            }
            esc.push(self.current_char.unwrap());
            self.advance();
        }
        match char::from_u32(u32::from_str_radix(&esc, if is_hex { 16 } else { 8 }).unwrap()) {
            Some(char) => Ok(char),
            None => lex_error!(self, start_pos, "Invalid character escape"),
        }
    }

    fn make_number(&mut self) -> Token<'f> {
        let start_location = self.location;
        let mut number = String::new();
        number.push(self.current_char.unwrap());
        self.advance();

        while self.current_char != None
            && (DIGITS.contains(&self.current_char.unwrap()) || self.current_char.unwrap() == '_')
        {
            if self.current_char.unwrap() != '_' {
                number.push(self.current_char.unwrap());
            }
            self.advance();
        }

        let next_char = self.next_char();
        if self.current_char == Some('.')
            && next_char != None
            && DIGITS.contains(&next_char.unwrap())
        {
            number.push('.');
            self.advance();
            number.push(next_char.unwrap());
            self.advance();

            while self.current_char != None
                && (DIGITS.contains(&self.current_char.unwrap())
                    || self.current_char.unwrap() == '_')
            {
                if self.current_char.unwrap() != '_' {
                    number.push(self.current_char.unwrap());
                }
                self.advance();
            }
        }

        Token::new(TokenType::Number, number, start_location, self.location)
    }

    fn make_dot(&mut self) -> LexResult<'f, Token> {
        let start_location = self.location;
        self.advance();

        if self.current_char != None && DIGITS.contains(&self.current_char.unwrap()) {
            let mut number = String::from("0.");
            number.push(self.current_char.unwrap());
            self.advance();

            while self.current_char != None
                && (DIGITS.contains(&self.current_char.unwrap())
                    || self.current_char.unwrap() == '_')
            {
                if self.current_char.unwrap() != '_' {
                    number.push(self.current_char.unwrap());
                }
                self.advance();
            }

            return Ok(Token::new(
                TokenType::Number,
                number,
                start_location,
                self.location,
            ));
        }

        if self.current_char == Some('.') {
            self.advance();
            if self.current_char == Some('=') {
                self.advance();
                return Ok(Token::new(
                    TokenType::DotsInclusive,
                    "..=".to_string(),
                    start_location,
                    self.location,
                ));
            }
            return Ok(Token::new(
                TokenType::Dots,
                "..".to_string(),
                start_location,
                self.location,
            ));
        }

        return Ok(Token::new(
            TokenType::Dot,
            ".".to_string(),
            start_location,
            self.location,
        ));
    }

    fn make_slash(&mut self) -> Option<Token<'f>> {
        let start_location = self.location;
        self.advance();
        match self.current_char {
            Some('=') => {
                self.advance();
                return Some(Token::new(
                    TokenType::DivideAssign,
                    "/=".to_string(),
                    start_location,
                    self.location,
                ));
            }
            Some('/') => {
                while ![Some('\n'), None].contains(&self.current_char) {
                    self.advance();
                }
                None
            }
            Some('|') => {
                self.advance();
                while let Some(current_char) = self.current_char {
                    if current_char == '|' && self.next_char() == Some('/') {
                        break;
                    }
                    self.advance();
                }
                self.advance();
                self.advance();
                None
            }
            _ => {
                return Some(Token::new(
                    TokenType::Divide,
                    "/".to_string(),
                    start_location,
                    self.location,
                ));
            }
        }
    }

    fn make_name(&mut self) -> Token<'f> {
        let start_location = self.location;
        let mut name = String::from(self.current_char.unwrap());
        self.advance();

        while self.current_char != None
            && (LETTERS.contains(&self.current_char.unwrap())
                || DIGITS.contains(&self.current_char.unwrap()))
        {
            name.push(self.current_char.unwrap());
            self.advance();
        }

        let token_type = match name.as_str() {
            "var" => TokenType::Var,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "null" => TokenType::Null,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "fun" => TokenType::Fun,
            "static" => TokenType::Static,
            "class" => TokenType::Class,
            "loop" => TokenType::Loop,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "in" => TokenType::In,
            "return" => TokenType::Return,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            _ => TokenType::Identifier,
        };
        Token::new(token_type, name, start_location, self.location)
    }
}
