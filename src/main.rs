#[macro_use]
mod error;
mod tokens;
mod lexer;
mod parser;
mod nodes;
mod interpreter;

use std::io::{Read, Write};
use interpreter::Interpreter;
use parser::Parser;
use lexer::Lexer;

fn main() {
    let start_total = std::time::Instant::now();

    let mut code = String::new();
    let mut file = std::fs::File::open(std::env::args().skip(1).next().unwrap_or(String::from("samples/sample.ro"))).unwrap();
    file.read_to_string(&mut code).unwrap();

    let end_read = start_total.elapsed();
    let start = std::time::Instant::now();

    let mut lexer = Lexer::new(&code);
    let tokens = match lexer.scan() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        },
    };
    file = std::fs::File::create("tokens.txt").unwrap();
    write!(file, "{:#?}", tokens).unwrap();

    let end_lex = start.elapsed();
    let start = std::time::Instant::now();

    let mut parser = Parser::new(&tokens);
    let nodes = match parser.parse() {
        Ok(nodes) => nodes,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        },
    };
    file = std::fs::File::create("nodes.txt").unwrap();
    write!(file, "{:#?}", nodes).unwrap();

    let end_parse = start.elapsed();
    let start = std::time::Instant::now();

    let mut interpreter = Interpreter::new(nodes);
    interpreter.run().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    let end_run = start.elapsed();
    let end = start_total.elapsed();
    println!(
        "\n\nRead File: {:?}\nScan tokens: {:?}\nParse AST: {:?}\nRun: {:?}\nTotal: {:?}",
        end_read,
        end_lex,
        end_parse,
        end_run,
        end,
    );
}
