mod tokens;
mod lexer;

use std::io::{Read, Write};
use crate::lexer::Lexer;

fn main() {
    let mut code = String::new();
    let mut file = std::fs::File::open("samples/sample.ro").unwrap();
    file.read_to_string(&mut code).unwrap();

    let mut lexer = Lexer::new(code);
    file = std::fs::File::create("tokens.txt").unwrap();
    write!(file, "{:#?}", lexer.scan()).unwrap();
}
