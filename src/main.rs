use std::io::{Read, Write};
use roost::{lexer::Lexer, parser::Parser, interpreter::Interpreter};

macro_rules! exit {
    ($error:expr, $code:expr) => {{
        let lines: Vec<&str> = $code.split('\n').collect();

        let line1 = if $error.location.line > 1 {
            format!("\n \x1b[90m{: >3} | \x1b[0m{}", $error.location.line - 1, lines[$error.location.line - 2])
        } else { String::new() };
        let line2 = format!(" \x1b[90m{: >3} | \x1b[0m{}", $error.location.line, lines[$error.location.line - 1]);
        let line3 = if $error.location.line < lines.len() {
            format!("\n \x1b[90m{: >3} | \x1b[0m{}", $error.location.line + 1, lines[$error.location.line])
        } else { String::new() };

        let marker = format!("{}\x1b[1;31m^\x1b[0m", " ".repeat($error.location.column + 6));

        eprintln!(
            "\x1b[1;36m{:?}\x1b[39m at {}:{}:{}\x1b[0m\n{}\n{}\n{}{}\n\n\x1b[1m{}\x1b[0m",
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

    let mut interpreter = Interpreter::new(nodes, |m| print!("{}", m));
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
