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

macro_rules! exit {
    ($error:expr, $code:expr) => {{
        let lines: Vec<&str> = $code.split('\n').collect();

        let line1 = if $error.location.line > 1 {
            format!("\n {: >3} | {}", $error.location.line - 1, lines[$error.location.line - 2])
        } else { String::new() };
        let line2 = format!(" {: >3} | {}", $error.location.line, lines[$error.location.line - 1]);
        let line3 = if $error.location.line < lines.len() {
            format!("\n {: >3} | {}", $error.location.line + 1, lines[$error.location.line])
        } else { String::new() };

        let marker = format!("{}^", " ".repeat($error.location.column + 6));

        eprintln!(
            "{:?} at {}:{}:{}\n{}\n{}\n{}{}\n\n{}",
            $error.kind,
            $error.location.filename,
            $error.location.line,
            $error.location.column,
            line1,
            line2,
            marker,
            line3,
            $error.message,
        );
        std::process::exit(1);
    }};
}

fn main() {
    let start_total = std::time::Instant::now();
    let filename = std::env::args().skip(1).next().unwrap_or(String::from("samples/sample.ro"));

    let mut code = String::new();
    let mut file = std::fs::File::open(&filename).unwrap();
    file.read_to_string(&mut code).unwrap();

    let end_read = start_total.elapsed();
    let start = std::time::Instant::now();

    let mut lexer = Lexer::new(&code, filename);
    let tokens = match lexer.scan() {
        Ok(tokens) => tokens,
        Err(e) => exit!(e, code),
    };
    file = std::fs::File::create("tokens.txt").unwrap();
    write!(file, "{:#?}", tokens).unwrap();

    let end_lex = start.elapsed();
    let start = std::time::Instant::now();

    let mut parser = Parser::new(&tokens);
    let nodes = match parser.parse() {
        Ok(nodes) => nodes,
        Err(e) => exit!(e, code),
    };
    file = std::fs::File::create("nodes.txt").unwrap();
    write!(file, "{:#?}", nodes).unwrap();

    let end_parse = start.elapsed();
    let start = std::time::Instant::now();

    let mut interpreter = Interpreter::new(nodes);
    interpreter.run().unwrap_or_else(|e| exit!(e, code));

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
