use crate::{
    error::{Error, Location, Span},
    tokens::{Token, TokenKind},
};
use std::{
    mem,
    str::{self, Chars},
};

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
            error_val!(SyntaxError, ($start, $self.location), $($arg)*),
            Token::new(
                TokenKind::Unknown,
                "Unknown".to_string(),
                Span::new($start, $self.location),
            ),
        ))
    };
}

macro_rules! char_construct {
    ($self:ident, $kind_single:ident, $kind_with_eq:tt, $kind_double:tt, $kind_double_with_eq:tt $(,)?) => {
        return Ok($self.make_char_construct(
            TokenKind::$kind_single,
            char_construct!(@optional $kind_with_eq),
            char_construct!(@optional $kind_double),
            char_construct!(@optional $kind_double_with_eq),
        ))
    };
    (@optional _) => { None };
    (@optional $kind:ident) => { Some(TokenKind::$kind) };
}

type LexResult<T> = std::result::Result<T, (Error, Token)>;

#[derive(Debug, Clone)]
pub struct Lexer<'i> {
    input: Chars<'i>,
    curr_char: Option<char>,
    next_char: Option<char>,
    location: Location,
}

impl<'i> Lexer<'i> {
    pub fn new(input: &'i str) -> Self {
        let mut lexer = Lexer {
            input: input.chars(),
            curr_char: None,
            next_char: None,
            location: Location::new(),
        };
        lexer.advance();
        lexer.advance();
        lexer
    }

    pub fn next_token(&mut self) -> LexResult<Token> {
        while let Some(curr_char) = self.curr_char {
            match curr_char {
                ' ' | '\t' | '\r' => self.advance(),
                '\n' => {
                    let start = self.location;
                    self.advance();
                    return Ok(Token::new(
                        TokenKind::Eol,
                        "LF".to_string(),
                        Span::new(start, self.location),
                    ));
                }
                '"' | '\'' => return self.make_string(),
                '.' => return self.make_dot(),
                '/' => {
                    if let Some(token) = self.make_slash() {
                        return Ok(token);
                    }
                }
                '*' => char_construct!(self, Star, StarAssign, Pow, PowAssign),
                '(' => char_construct!(self, LParen, _, _, _),
                ')' => char_construct!(self, RParen, _, _, _),
                '{' => char_construct!(self, LBrace, _, _, _),
                '}' => char_construct!(self, RBrace, _, _, _),
                '[' => char_construct!(self, LBrack, _, _, _),
                ']' => char_construct!(self, RBrack, _, _, _),
                ',' => char_construct!(self, Comma, _, _, _),
                ';' => char_construct!(self, Semicolon, _, _, _),
                '|' => char_construct!(self, BitOr, BitOrAssign, Or, _),
                '^' => char_construct!(self, BitXor, BitXorAssign, _, _),
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
                '%' => char_construct!(self, Rem, RemAssign, _, _),
                '\\' => char_construct!(self, Backslash, BackslashAssign, _, _),
                _ => {
                    if DIGITS.contains(&curr_char) {
                        return Ok(self.make_number());
                    } else if LETTERS.contains(&curr_char) {
                        return Ok(self.make_name());
                    } else {
                        let start_pos = self.location;
                        self.advance();
                        lex_error!(self, start_pos, "Illegal character '{}'", curr_char);
                    }
                }
            }
        }

        let start = self.location;
        self.location.advance(false);
        Ok(Token::new(
            TokenKind::Eof,
            "EOF".to_string(),
            Span::new(start, self.location),
        ))
    }

    fn advance(&mut self) {
        if let Some(curr_char) = self.curr_char {
            self.location.advance(curr_char == '\n');
        }
        mem::swap(&mut self.curr_char, &mut self.next_char);
        self.next_char = self.input.next();
    }

    // ----------------------------------------

    fn make_char_construct(
        &mut self,
        kind_single: TokenKind,
        kind_with_eq: Option<TokenKind>,
        kind_double: Option<TokenKind>,
        kind_double_with_eq: Option<TokenKind>,
    ) -> Token {
        let start = self.location;
        let char = self.curr_char.unwrap();
        self.advance();
        match (
            kind_with_eq,
            &kind_double,
            &kind_double_with_eq,
            self.curr_char,
        ) {
            (Some(ty), .., Some('=')) => {
                self.advance();
                Token::new(ty, char.to_string() + "=", Span::new(start, self.location))
            }
            (_, Some(_), _, Some(c)) | (_, _, Some(_), Some(c)) if c == char => {
                self.advance();
                match (kind_double, kind_double_with_eq, self.curr_char) {
                    (_, Some(ty), Some('=')) => {
                        self.advance();
                        Token::new(
                            ty,
                            format!("{char}{char}="),
                            Span::new(start, self.location),
                        )
                    }
                    (Some(ty), ..) => {
                        Token::new(ty, format!("{char}{char}"), Span::new(start, self.location))
                    }
                    // can panic when all this is true:
                    // - `kind_double` is `None`
                    // - `kind_double_with_eq` is `Some(_)`
                    // - `self.curr_char` is not `Some('=')`
                    // but we never call this function that way
                    _ => unreachable!(),
                }
            }
            _ => Token::new(
                kind_single,
                char.to_string(),
                Span::new(start, self.location),
            ),
        }
    }

    fn make_string(&mut self) -> LexResult<Token> {
        let start = self.location;
        let start_quote = self.curr_char;
        let mut string = String::new();

        self.advance(); // start quote
        while ![start_quote, Some('\\'), None].contains(&self.curr_char) {
            string.push(self.curr_char.unwrap());
            self.advance();
        }
        while self.curr_char == Some('\\') {
            let escape_pos = self.location;
            self.advance(); // backslash
            if self.curr_char == None {
                lex_error!(self, escape_pos, "Invalid escape sequence")
            }
            let curr_char = self.curr_char.unwrap();

            if ESCAPE_CHAR.contains(&curr_char) {
                string.push(match curr_char {
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
            } else if OCTAL_DIGITS.contains(&curr_char) {
                string.push(self.escape_sequence(&curr_char, escape_pos, false, 2)?);
            } else if curr_char == 'x' {
                string.push(self.escape_sequence(&curr_char, escape_pos, true, 2)?);
            } else if curr_char == 'u' {
                string.push(self.escape_sequence(&curr_char, escape_pos, true, 4)?);
            } else if curr_char == 'U' {
                string.push(self.escape_sequence(&curr_char, escape_pos, true, 8)?);
            } else {
                lex_error!(self, escape_pos, "Invalid escape sequence");
            }

            while ![start_quote, Some('\\'), None].contains(&self.curr_char) {
                string.push(self.curr_char.unwrap());
                self.advance();
            }
        }
        self.advance(); // end quote

        Ok(Token::new(
            TokenKind::String,
            string,
            Span::new(start, self.location),
        ))
    }

    fn escape_sequence(
        &mut self,
        curr_char: &char,
        start_pos: Location,
        is_hex: bool,
        digits: u8,
    ) -> LexResult<char> {
        let mut esc = if is_hex {
            String::new()
        } else {
            curr_char.to_string()
        };
        self.advance();
        for _ in 0..digits {
            if self.curr_char == None
                || if is_hex {
                    !HEX_DIGITS.contains(&self.curr_char.unwrap())
                } else {
                    !OCTAL_DIGITS.contains(&self.curr_char.unwrap())
                }
            {
                lex_error!(self, start_pos, "Invalid escape sequence");
            }
            esc.push(self.curr_char.unwrap());
            self.advance();
        }
        match char::from_u32(u32::from_str_radix(&esc, if is_hex { 16 } else { 8 }).unwrap()) {
            Some(char) => Ok(char),
            None => lex_error!(self, start_pos, "Invalid character escape"),
        }
    }

    fn make_number(&mut self) -> Token {
        let start = self.location;
        let mut number = String::new();
        number.push(self.curr_char.unwrap());
        self.advance();

        while self.curr_char != None
            && (DIGITS.contains(&self.curr_char.unwrap()) || self.curr_char.unwrap() == '_')
        {
            if self.curr_char.unwrap() != '_' {
                number.push(self.curr_char.unwrap());
            }
            self.advance();
        }

        let next_char = self.next_char;
        if self.curr_char == Some('.') && next_char != None && DIGITS.contains(&next_char.unwrap())
        {
            number.push('.');
            self.advance();
            number.push(next_char.unwrap());
            self.advance();

            while self.curr_char != None
                && (DIGITS.contains(&self.curr_char.unwrap()) || self.curr_char.unwrap() == '_')
            {
                if self.curr_char.unwrap() != '_' {
                    number.push(self.curr_char.unwrap());
                }
                self.advance();
            }
        }

        Token::new(TokenKind::Number, number, Span::new(start, self.location))
    }

    fn make_dot(&mut self) -> LexResult<Token> {
        let start = self.location;
        self.advance();

        if self.curr_char != None && DIGITS.contains(&self.curr_char.unwrap()) {
            let mut number = String::from("0.");
            number.push(self.curr_char.unwrap());
            self.advance();

            while self.curr_char != None
                && (DIGITS.contains(&self.curr_char.unwrap()) || self.curr_char.unwrap() == '_')
            {
                if self.curr_char.unwrap() != '_' {
                    number.push(self.curr_char.unwrap());
                }
                self.advance();
            }

            return Ok(Token::new(
                TokenKind::Number,
                number,
                Span::new(start, self.location),
            ));
        }

        if self.curr_char == Some('.') {
            self.advance();
            if self.curr_char == Some('=') {
                self.advance();
                return Ok(Token::new(
                    TokenKind::DotsInclusive,
                    "..=".to_string(),
                    Span::new(start, self.location),
                ));
            }
            return Ok(Token::new(
                TokenKind::Dots,
                "..".to_string(),
                Span::new(start, self.location),
            ));
        }

        Ok(Token::new(
            TokenKind::Dot,
            ".".to_string(),
            Span::new(start, self.location),
        ))
    }

    fn make_slash(&mut self) -> Option<Token> {
        let start = self.location;
        self.advance();
        match self.curr_char {
            Some('=') => {
                self.advance();
                Some(Token::new(
                    TokenKind::SlashAssign,
                    "/=".to_string(),
                    Span::new(start, self.location),
                ))
            }
            Some('/') => {
                while ![Some('\n'), None].contains(&self.curr_char) {
                    self.advance();
                }
                None
            }
            Some('|') => {
                self.advance();
                while let Some(curr_char) = self.curr_char {
                    if curr_char == '|' && self.next_char == Some('/') {
                        break;
                    }
                    self.advance();
                }
                self.advance();
                self.advance();
                None
            }
            _ => Some(Token::new(
                TokenKind::Slash,
                "/".to_string(),
                Span::new(start, self.location),
            )),
        }
    }

    fn make_name(&mut self) -> Token {
        let start = self.location;
        let mut name = String::from(self.curr_char.unwrap());
        self.advance();

        while self.curr_char != None
            && (LETTERS.contains(&self.curr_char.unwrap())
                || DIGITS.contains(&self.curr_char.unwrap()))
        {
            name.push(self.curr_char.unwrap());
            self.advance();
        }

        let kind = match name.as_str() {
            "var" => TokenKind::Var,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "fun" => TokenKind::Fun,
            "static" => TokenKind::Static,
            "class" => TokenKind::Class,
            "loop" => TokenKind::Loop,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "return" => TokenKind::Return,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "try" => TokenKind::Try,
            "catch" => TokenKind::Catch,
            _ => TokenKind::Identifier,
        };
        Token::new(kind, name, Span::new(start, self.location))
    }
}
