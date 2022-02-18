use std::str::Chars;
use crate::{tokens::{Token, TokenType}, error::{Location, Error}};

const DIGITS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
const OCTAL_DIGITS: [char; 8] = ['0', '1', '2', '3', '4', '5', '6', '7'];
const HEX_DIGITS: [char; 22] = ['0', '1', '2', '3', '4', '5', '6', '7', '8',
    '9', 'A', 'a', 'B', 'b', 'C', 'c', 'D', 'd', 'E', 'e', 'F', 'f'];
const LETTERS_AND_UNDERSCORE: [char; 53] = ['A', 'a', 'B', 'b', 'C', 'c', 'D',
    'd', 'E', 'e', 'F', 'f', 'G', 'g', 'H', 'h', 'I', 'i', 'J', 'j', 'K', 'k',
    'L', 'l', 'M', 'm', 'N', 'n', 'O', 'o', 'P', 'p', 'Q', 'q', 'R', 'r', 'S',
    's', 'T', 't', 'U', 'u', 'V', 'v', 'W', 'w', 'X', 'x', 'Y', 'y', 'Z', 'z', '_'];

const ESCAPE_CHAR: [char; 10] = ['\\', '\'', '"', 'a', 'b', 'f', 'n', 'r', 't', 'v'];

macro_rules! loc {
    ($self:ident) => {
        $self.location.clone()
    };
}

macro_rules! lex_error {
    ($self:ident, $start:ident, $($arg:tt)*) => {
        return Err((
            error_val!(SyntaxError, $start.clone(), loc!($self), $($arg)*),
            Token::new(
                TokenType::Unknown,
                "Unknown",
                $start,
                loc!($self),
            ),
        ))
    };
}

type LexResult<T> = std::result::Result<T, (Error, Token)>;

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    input: Chars<'a>,
    current_char: Option<char>,
    location: Location,
}

impl <'a> Lexer<'a> {
    pub fn new(input: &'a String, filename: String) -> Self {
        let mut lexer = Lexer {
            input: input.chars(),
            current_char: None,
            location: Location::new(filename),
        };
        lexer.advance();
        return lexer;
    }

    pub fn next_token(&mut self) -> LexResult<Token> {
        while let Some(current_char) = self.current_char {
            match current_char {
                ' ' | '\t' | '\r' => self.advance(),
                '"' | '\'' => return self.make_string(),
                '.' => return self.make_dot(),
                '/' => { let token = self.make_slash(); if let Some(token) = token { return Ok(token) } },
                '*' => return Ok(self.make_star()),
                '(' => return Ok(self.make_single_char(TokenType::LParen,       "(")),
                ')' => return Ok(self.make_single_char(TokenType::RParen,       ")")),
                '{' => return Ok(self.make_single_char(TokenType::LBrace,       "{")),
                '}' => return Ok(self.make_single_char(TokenType::RBrace,       "}")),
                '|' => return Ok(self.make_single_char(TokenType::Or,           "|")),
                '&' => return Ok(self.make_single_char(TokenType::And,          "&")),
                ',' => return Ok(self.make_single_char(TokenType::Comma,        ",")),
                ';' => return Ok(self.make_single_char(TokenType::EOL,          ";")),
                '\n' => return Ok(self.make_single_char(TokenType::EOL,         "LF")),
                '=' => return Ok(self.make_optional_equal(TokenType::Assign,      TokenType::Equal,              "=")),
                '!' => return Ok(self.make_optional_equal(TokenType::Not,         TokenType::NotEqual,           "!")),
                '<' => return Ok(self.make_optional_equal(TokenType::LessThan,    TokenType::LessThanOrEqual,    "<")),
                '>' => return Ok(self.make_optional_equal(TokenType::GreaterThan, TokenType::GreaterThanOrEqual, ">")),
                '+' => return Ok(self.make_optional_equal(TokenType::Plus,        TokenType::PlusAssign,         "+")),
                '-' => return Ok(self.make_optional_equal(TokenType::Minus,       TokenType::MinusAssign,        "-")),
                '%' => return Ok(self.make_optional_equal(TokenType::Modulo,      TokenType::ModuloAssign,       "%")),
                '\\' => return Ok(self.make_optional_equal(TokenType::IntDivide,  TokenType::IntDivideAssign,    "\\")),
                _ => {
                    if DIGITS.contains(&current_char) {
                        return Ok(self.make_number());
                    } else if LETTERS_AND_UNDERSCORE.contains(&current_char) {
                        return Ok(self.make_name());
                    } else {
                        let start_pos = loc!(self);
                        self.advance();
                        lex_error!(self, start_pos, "Illegal character '{}'", current_char);
                    }
                },
            }
        }

        let start_pos = loc!(self);
        self.location.advance(false);
        return Ok(Token::new(TokenType::EOF, "EOF", start_pos, loc!(self)));
    }

    fn advance(&mut self) {
        if let Some(current_char) = self.current_char {
            self.location.advance(current_char == '\n')
        }
        self.current_char = self.input.next();
    }

    fn next_char(&self) -> Option<char> {
        return self.input.clone().next();
    }

    // ----------------------------------------

    fn make_single_char(&mut self, token_type: TokenType, value: &str) -> Token {
        let start_location = loc!(self);
        self.advance();
        return Token::new(
            token_type,
            value,
            start_location,
            loc!(self),
        );
    }

    fn make_string(&mut self) -> LexResult<Token> {
        let start_location = loc!(self);
        let start_quote = self.current_char;
        let mut string = String::new();

        self.advance(); // start quote
        while ![start_quote, Some('\\'), None].contains(&self.current_char) {
            string.push(self.current_char.unwrap());
            self.advance();
        }
        while self.current_char == Some('\\') {
            let escape_pos = loc!(self);
            self.advance(); // backslash
            if self.current_char == None { lex_error!(self, escape_pos, "Invalid escape sequence") }
            let current_char = self.current_char.unwrap();

            if ESCAPE_CHAR.contains(&current_char) {
                string.push(match current_char {
                    '\\' => '\\',
                    '\'' => '\'',
                    '"'  => '"',
                    'a'  => '\x07',
                    'b'  => '\x08',
                    'f'  => '\x0c',
                    'n'  => '\n',
                    'r'  => '\r',
                    't'  => '\t',
                    'v'  => '\x0b',
                    _ => unreachable!(),
                });
                self.advance();
            } else if OCTAL_DIGITS.contains(&current_char) {
                string.push(self.escape_sequence(
                    &current_char,
                    escape_pos,
                    false,
                    2,
                )?);
            } else if current_char == 'x' {
                string.push(self.escape_sequence(
                    &current_char,
                    escape_pos,
                    true,
                    2,
                )?);
            } else if current_char == 'u' {
                string.push(self.escape_sequence(
                    &current_char,
                    escape_pos,
                    true,
                    4,
                )?);
            } else if current_char == 'U' {
                string.push(self.escape_sequence(
                    &current_char,
                    escape_pos,
                    true,
                    8,
                )?);
            } else {
                lex_error!(self, escape_pos, "Invalid escape sequence");
            }

            while ![start_quote, Some('\\'), None].contains(&self.current_char) {
                string.push(self.current_char.unwrap());
                self.advance();
            }
        }
        self.advance(); // end quote

        return Ok(Token::new(TokenType::String, &string, start_location, loc!(self)));
    }

    fn escape_sequence(&mut self, current_char: &char, start_pos: Location, is_hex: bool, digits: u8) -> LexResult<char> {
        let mut esc = if is_hex { String::new() } else { current_char.to_string() };
        self.advance();
        for _ in 0..digits {
            if self.current_char == None || if is_hex {
                !HEX_DIGITS.contains(&self.current_char.unwrap())
            } else {
                !OCTAL_DIGITS.contains(&self.current_char.unwrap())
            } {
                lex_error!(self, start_pos, "Invalid escape sequence");
            }
            esc.push(self.current_char.unwrap());
            self.advance();
        }
        match char::from_u32(u32::from_str_radix(&esc, if is_hex { 16 } else { 8 }).unwrap()) {
            Some(char) => return Ok(char),
            None       => lex_error!(self, start_pos, "Invalid character escape"),
        }
    }

    fn make_number(&mut self) -> Token {
        let start_location = loc!(self);
        let mut number = String::new();
        number.push(self.current_char.unwrap());
        self.advance();

        while self.current_char != None && (DIGITS.contains(&self.current_char.unwrap()) || self.current_char.unwrap() == '_') {
            if self.current_char.unwrap() != '_' { number.push(self.current_char.unwrap()); }
            self.advance();
        }

        let next_char = self.next_char();
        if self.current_char == Some('.') && next_char != None && DIGITS.contains(&next_char.unwrap()) {
            number.push('.');
            self.advance();
            number.push(next_char.unwrap());
            self.advance();

            while self.current_char != None && (DIGITS.contains(&self.current_char.unwrap()) || self.current_char.unwrap() == '_') {
                if self.current_char.unwrap() != '_' { number.push(self.current_char.unwrap()); }
                self.advance();
            }
        }

        return Token::new(TokenType::Number, &number, start_location, loc!(self));
    }

    fn make_dot(&mut self) -> LexResult<Token> {
        let start_location = loc!(self);
        self.advance();

        if self.current_char != None && DIGITS.contains(&self.current_char.unwrap()) {
            let mut number = String::from("0.");
            number.push(self.current_char.unwrap());
            self.advance();

            while self.current_char != None && (DIGITS.contains(&self.current_char.unwrap()) || self.current_char.unwrap() == '_') {
                if self.current_char.unwrap() != '_' { number.push(self.current_char.unwrap()); }
                self.advance();
            }

            return Ok(Token::new(TokenType::Number, &number, start_location, loc!(self)));
        }

        if self.current_char == Some('.') {
            self.advance();
            if self.current_char == Some('=') {
                self.advance();
                return Ok(Token::new(TokenType::RangeDots, "..=", start_location, loc!(self)));
            }
            return Ok(Token::new(TokenType::RangeDots, "..", start_location, loc!(self)));
        }

        return Ok(Token::new(TokenType::Dot, ".", start_location, loc!(self)));
    }

    fn make_optional_equal(&mut self, type_single: TokenType, type_with_eq: TokenType, base_value: &str) -> Token {
        let start_location = loc!(self);
        self.advance();
        if self.current_char == Some('=') {
            self.advance();
            return Token::new(type_with_eq, &(base_value.to_string() + "="), start_location, loc!(self));
        }
        return Token::new(type_single, base_value, start_location, loc!(self));
    }

    fn make_star(&mut self) -> Token {
        let start_location = loc!(self);
        self.advance();
        if self.current_char == Some('*') {
            self.advance();
            if self.current_char == Some('=') {
                self.advance();
                return Token::new(TokenType::PowerAssign, "**=", start_location, loc!(self));
            }
            return Token::new(TokenType::Power, "**", start_location, loc!(self));
        } else if self.current_char == Some('=') {
            self.advance();
            return Token::new(TokenType::MultiplyAssign, "*=", start_location, loc!(self));
        }
        return Token::new(TokenType::Multiply, "*", start_location, loc!(self));
    }

    fn make_slash(&mut self) -> Option<Token> {
        let start_location = loc!(self);
        self.advance();
        match self.current_char {
            Some('=') => {
                self.advance();
                return Some(Token::new(TokenType::DivideAssign, "/=", start_location, loc!(self)));
            },
            Some('/') => {
                while ![Some('\n'), None].contains(&self.current_char) {
                    self.advance();
                }
                return None;
            },
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
                return None;
            },
            _ => {
                return Some(Token::new(TokenType::Divide, "/", start_location, loc!(self)));
            }
        }
    }

    fn make_name(&mut self) -> Token {
        let start_location = loc!(self);
        let mut name = String::from(self.current_char.unwrap());
        self.advance();

        while self.current_char != None && (
            LETTERS_AND_UNDERSCORE.contains(&self.current_char.unwrap())
            || DIGITS.contains(&self.current_char.unwrap())
        ) {
            name.push(self.current_char.unwrap());
            self.advance();
        }

        let token_type =  match name.as_str() {
            "var"      => TokenType::Var,
            "true"     => TokenType::True,
            "false"    => TokenType::False,
            "null"     => TokenType::Null,
            "if"       => TokenType::If,
            "else"     => TokenType::Else,
            "fun"      => TokenType::Fun,
            "static"   => TokenType::Static,
            "class"    => TokenType::Class,
            "loop"     => TokenType::Loop,
            "while"    => TokenType::While,
            "for"      => TokenType::For,
            "in"       => TokenType::In,
            "return"   => TokenType::Return,
            "break"    => TokenType::Break,
            "continue" => TokenType::Continue,
            _ => TokenType::Identifier,
        };
        return Token::new(token_type, &name, start_location, loc!(self));
    }
}
