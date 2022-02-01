use std::str::Chars;
use crate::{tokens::{Token, TokenType}, error::{Result, Location}};

const DIGITS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
const OCTAL_DIGITS: [char; 8] = ['0', '1', '2', '3', '4', '5', '6', '7'];
const HEX_DIGITS: [char; 22] = ['0', '1', '2', '3', '4', '5', '6', '7', '8',
    '9', 'A', 'a', 'B', 'b', 'C', 'c', 'D', 'd', 'E', 'e', 'F', 'f'];
const LETTERS_AND_UNDERSCORE: [char; 53] = ['A', 'a', 'B', 'b', 'C', 'c', 'D',
    'd', 'E', 'e', 'F', 'f', 'G', 'g', 'H', 'h', 'I', 'i', 'J', 'j', 'K', 'k',
    'L', 'l', 'M', 'm', 'N', 'n', 'O', 'o', 'P', 'p', 'Q', 'q', 'R', 'r', 'S',
    's', 'T', 't', 'U', 'u', 'V', 'v', 'W', 'w', 'X', 'x', 'Y', 'y', 'Z', 'z', '_'];
const SPACES: [char; 3] = [' ', '\t', '\r'];

const SINGLE_CHARS: [char; 11] = ['(', ')', '{', '}', '?', ':', '|', '&', ',', '\n', ';'];
const OPTIONAL_EQ_CHARS: [char; 8] = ['=', '!', '<', '>', '+', '-', '%', '\\'];
const KEYWORDS: [&str; 14] = ["var", "true", "false", "if", "null", "else", "fun",
    "loop", "while", "for", "in", "return", "break", "continue"];
const ESCAPE_CHAR: [char; 10] = ['\\', '\'', '"', 'a', 'b', 'f', 'n', 'r', 't', 'v'];

#[derive(Debug)]
pub struct Lexer<'a> {
    input: Chars<'a>,
    current_char: Option<char>,
    location: Location,
}

impl <'a> Lexer<'a> {
    pub fn new(input: &'a String, filename: String) -> Self {
        return Lexer {
            input: input.chars(),
            current_char: None,
            location: Location::new(filename),
        };
    }

    pub fn scan(&mut self) -> Result<Vec<Token>> {
        self.advance();
        let mut tokens: Vec<Token> = vec![];

        while let Some(current_char) = self.current_char {
            if SPACES.contains(&current_char) {
                self.advance();
            } else if SINGLE_CHARS.contains(&current_char) {
                tokens.push(self.make_single_char());
            } else if ['"', '\''].contains(&current_char) {
                tokens.push(self.make_string()?);
            } else if DIGITS.contains(&current_char) {
                tokens.push(self.make_number());
            } else if current_char == '.' {
                tokens.push(self.make_dot()?);
            } else if OPTIONAL_EQ_CHARS.contains(&current_char) {
                tokens.push(self.make_optional_equal());
            } else if current_char == '/' {
                let token = self.make_slash();
                if let Some(token) = token { tokens.push(token) }
            } else if current_char == '*' {
                tokens.push(self.make_star());
            } else if LETTERS_AND_UNDERSCORE.contains(&current_char) {
                tokens.push(self.make_name());
            } else {
                error!(SyntaxError, self.location.clone(), "Illegal character '{}'", current_char);
            }
        }
        tokens.push(Token::new(TokenType::EOF, "EOF", self.location.clone()));

        return Ok(tokens);
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

    fn make_single_char(&mut self) -> Token {
        let start_location = self.location.clone();
        let char = self.current_char.unwrap();
        let token_type = match char {
            '('  => TokenType::LParen,
            ')'  => TokenType::RParen,
            '{'  => TokenType::LBrace,
            '}'  => TokenType::RBrace,
            '?'  => TokenType::QuestionMark,
            ':'  => TokenType::Colon,
            '|'  => TokenType::Or,
            '&'  => TokenType::And,
            ','  => TokenType::Comma,
            '\n' | ';' => TokenType::EOL,
            _ => panic!(),
        };
        self.advance();
        let value = char.to_string();
        return Token::new(
            token_type,
            if char == '\n' { "LF" } else { &value },
            start_location,
        );
    }

    fn make_string(&mut self) -> Result<Token> {
        let start_location = self.location.clone();
        let start_quote = self.current_char;
        let mut string = String::new();

        self.advance(); // start quote
        while ![start_quote, Some('\\'), None].contains(&self.current_char) {
            string.push(self.current_char.unwrap());
            self.advance();
        }
        while self.current_char == Some('\\') {
            let escape_pos = self.location.clone();
            self.advance(); // backslash
            if self.current_char == None { error!(SyntaxError, escape_pos, "Invalid escape sequence") }
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
                    _ => panic!(),
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
                error!(SyntaxError, escape_pos, "Invalid escape sequence");
            }

            while ![start_quote, Some('\\'), None].contains(&self.current_char) {
                string.push(self.current_char.unwrap());
                self.advance();
            }
        }
        self.advance(); // end quote

        return Ok(Token::new(TokenType::String, &string, start_location));
    }

    fn escape_sequence(&mut self, current_char: &char, start_pos: Location, is_hex: bool, digits: u8) -> Result<char> {
        let mut esc = if is_hex { String::new() } else { current_char.to_string() };
        self.advance();
        for _ in 0..digits {
            if self.current_char == None || if is_hex {
                !HEX_DIGITS.contains(&self.current_char.unwrap())
            } else {
                !OCTAL_DIGITS.contains(&self.current_char.unwrap())
            } {
                error!(SyntaxError, start_pos, "Invalid escape sequence");
            }
            esc.push(self.current_char.unwrap());
            self.advance();
        }
        match char::from_u32(u32::from_str_radix(&esc, if is_hex { 16 } else { 8 }).unwrap()) {
            Some(char) => return Ok(char),
            None       => error!(SyntaxError, start_pos, "Invalid character escape"),
        }
    }

    fn make_number(&mut self) -> Token {
        let start_location = self.location.clone();
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

        return Token::new(TokenType::Number, &number, start_location);
    }

    fn make_dot(&mut self) -> Result<Token> {
        let start_location = self.location.clone();
        self.advance();

        if self.current_char != Some('.') {
            if let Some(current_char) = self.current_char {
                error!(SyntaxError, self.location.clone(), "Expected '.', got '{}'", current_char);
            }
            error!(SyntaxError, self.location.clone(), "Expected '.'");
        }

        self.advance();
        if self.current_char == Some('=') {
            self.advance();
            return Ok(Token::new(TokenType::RangeDots, "..=", start_location));
        }
        return Ok(Token::new(TokenType::RangeDots, "..", start_location));
    }

    fn make_optional_equal(&mut self) -> Token {
        let start_location = self.location.clone();
        let char = self.current_char.unwrap();
        let token_types = match char {
            '='  => (TokenType::Assign,      TokenType::Equal             ),
            '!'  => (TokenType::Not,         TokenType::NotEqual          ),
            '<'  => (TokenType::LessThan,    TokenType::LessThanOrEqual   ),
            '>'  => (TokenType::GreaterThan, TokenType::GreaterThanOrEqual),
            '+'  => (TokenType::Plus,        TokenType::PlusAssign        ),
            '-'  => (TokenType::Minus,       TokenType::MinusAssign       ),
            '%'  => (TokenType::Modulo,      TokenType::ModuloAssign      ),
            '\\' => (TokenType::IntDivide,   TokenType::IntDivideAssign   ),
            _ => panic!()
        };
        self.advance();
        if self.current_char == Some('=') {
            self.advance();
            return Token::new(token_types.1, &(char.to_string() + "="), start_location);
        }
        return Token::new(token_types.0, &char.to_string(), start_location);
    }

    fn make_star(&mut self) -> Token {
        let start_location = self.location.clone();
        self.advance();
        if self.current_char == Some('*') {
            self.advance();
            if self.current_char == Some('=') {
                self.advance();
                return Token::new(TokenType::PowerAssign, "**=", start_location);
            }
            return Token::new(TokenType::Power, "**", start_location);
        } else if self.current_char == Some('=') {
            self.advance();
            return Token::new(TokenType::MultiplyAssign, "*=", start_location);
        }
        return Token::new(TokenType::Multiply, "*", start_location);
    }

    fn make_slash(&mut self) -> Option<Token> {
        let start_location = self.location.clone();
        self.advance();
        match self.current_char {
            Some('=') => {
                self.advance();
                return Some(Token::new(TokenType::DivideAssign, "/=", start_location));
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
                return Some(Token::new(TokenType::Divide, "/", start_location));
            }
        }
    }

    fn make_name(&mut self) -> Token {
        let start_location = self.location.clone();
        let mut name = String::from(self.current_char.unwrap());
        self.advance();

        while self.current_char != None && (
            LETTERS_AND_UNDERSCORE.contains(&self.current_char.unwrap())
            || DIGITS.contains(&self.current_char.unwrap())
        ) {
            name.push(self.current_char.unwrap());
            self.advance();
        }

        if KEYWORDS.contains(&name.as_str()) {
            return Token::new(TokenType::Keyword, &name, start_location);
        }
        return Token::new(TokenType::Identifier, &name, start_location);
    }
}
