use std::str::Chars;
use crate::{tokens::{Token, TokenType}, error::Result};

const DIGITS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
const LETTERS_AND_UNDERSCORE: [char; 53] = ['A', 'a', 'B', 'b', 'C', 'c', 'D',
    'd', 'E', 'e', 'F', 'f', 'G', 'g', 'H', 'h', 'I', 'i', 'J', 'j', 'K', 'k',
    'L', 'l', 'M', 'm', 'N', 'n', 'O', 'o', 'P', 'p', 'Q', 'q', 'R', 'r', 'S',
    's', 'T', 't', 'U', 'u', 'V', 'v', 'W', 'w', 'X', 'x', 'Y', 'y', 'Z', 'z', '_'];
const SPACES: [char; 2] = [' ', '\t'];

const SINGLE_CHARS: [char; 11] = ['(', ')', '{', '}', '?', ':', '|', '&', ',', '\n', ';'];
const OPTIONAL_EQ_CHARS: [char; 6] = ['=', '!', '<', '>', '+', '-'];
const KEYWORDS: [&str; 14] = ["var", "true", "false", "if", "null", "else", "fun",
    "loop", "while", "for", "in", "return", "break", "continue"];

pub struct Lexer<'a> {
    input: Chars<'a>,
    current_char: Option<char>,
}

impl <'a> Lexer<'a> {
    pub fn new(input: &'a String) -> Self {
        return Lexer {
            input: input.chars(),
            current_char: None,
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
                tokens.push(self.make_string());
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
                error!(SyntaxError, "Illegal character `{}`", current_char);
            }
        }
        tokens.push(Token::new(TokenType::EOF, "EOF"));

        return Ok(tokens);
    }

    fn advance(&mut self) {
        self.current_char = self.input.next();
    }

    fn next_char(&self) -> Option<char> {
        return self.input.clone().next();
    }

    // ----------------------------------------

    fn make_single_char(&mut self) -> Token {
        let char = self.current_char.unwrap();
        let token_type = match char {
            '(' => TokenType::LParen,
            ')' => TokenType::RParen,
            '{' => TokenType::LBrace,
            '}' => TokenType::RBrace,
            '?' => TokenType::QuestionMark,
            ':' => TokenType::Colon,
            '|' => TokenType::Or,
            '&' => TokenType::And,
            ',' => TokenType::Comma,
            '\n' | ';' => TokenType::EOL,
            _ => panic!(),
        };
        self.advance();
        return Token::new(token_type, &char.to_string());
    }

    fn make_string(&mut self) -> Token {
        let start_quote = self.current_char;
        let mut string = String::new();

        self.advance(); // start quote
        while ![start_quote, None].contains(&self.current_char) {
            string.push(self.current_char.unwrap());
            self.advance();
        }
        self.advance(); // end quote

        string = string.replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\r", "\r");
        return Token::new(TokenType::String, &string);
    }

    fn make_number(&mut self) -> Token {
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

        return Token::new(TokenType::Number, &number);
    }

    fn make_dot(&mut self) -> Result<Token> {
        self.advance();

        if self.current_char != Some('.') {
            if let Some(current_char) = self.current_char {
                error!(SyntaxError, "Expected `.`, got `{}`", current_char);
            }
            error!(SyntaxError, "Expected `.`");
        }

        self.advance();
        if self.current_char == Some('=') {
            self.advance();
            return Ok(Token::new(TokenType::RangeDots, "..="));
        }
        return Ok(Token::new(TokenType::RangeDots, ".."));
    }

    fn make_optional_equal(&mut self) -> Token {
        let char = self.current_char.unwrap();
        let token_types = match char {
            '=' => (TokenType::Assign,      TokenType::Equal             ),
            '!' => (TokenType::Not,         TokenType::NotEqual          ),
            '<' => (TokenType::LessThan,    TokenType::LessThanOrEqual   ),
            '>' => (TokenType::GreaterThan, TokenType::GreaterThanOrEqual),
            '+' => (TokenType::Plus,        TokenType::PlusAssign        ),
            '-' => (TokenType::Minus,       TokenType::MinusAssign       ),
            _ => panic!()
        };
        self.advance();
        if self.current_char == Some('=') {
            self.advance();
            return Token::new(token_types.1, &(char.to_string() + "="));
        }
        return Token::new(token_types.0, &char.to_string());
    }

    fn make_star(&mut self) -> Token {
        self.advance();
        if self.current_char == Some('*') {
            self.advance();
            return Token::new(TokenType::Power, "**");
        }
        return Token::new(TokenType::Multiply, "*");
    }

    fn make_slash(&mut self) -> Option<Token> {
        self.advance();
        match self.current_char {
            Some('=') => {
                self.advance();
                return Some(Token::new(TokenType::DivideAssign, "/="));
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
                return Some(Token::new(TokenType::Divide, "/"));
            }
        }
    }

    fn make_name(&mut self) -> Token {
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
            return Token::new(TokenType::Keyword, &name);
        }
        return Token::new(TokenType::Identifier, &name);
    }
}
