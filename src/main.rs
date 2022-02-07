use std::{io::{Read, /* Write */}, time::Instant, fs::File};
use structopt::StructOpt;
use roost::{lexer::Lexer, parser::Parser, interpreter::Interpreter};

/// Command line interpreter for the roost language
#[derive(StructOpt)]
#[structopt(author)]
struct Roost {
    /// File to run
    #[structopt()]
    file: String,

    /// Measure and display the time of execution
    #[structopt(short, long)]
    time: bool,
}

macro_rules! print_error {
    ($error:expr, $code:expr) => {
        let lines: Vec<&str> = $code.split('\n').collect();

        let line1 = if $error.start.line > 1 {
            format!("\n \x1b[90m{: >3} | \x1b[0m{}", $error.start.line - 1, lines[$error.start.line - 2])
        } else { String::new() };
        let line2 = format!(" \x1b[90m{: >3} | \x1b[0m{}", $error.start.line, lines[$error.start.line - 1]);
        let line3 = if $error.start.line < lines.len() {
            format!("\n \x1b[90m{: >3} | \x1b[0m{}", $error.start.line + 1, lines[$error.start.line])
        } else { String::new() };

        let marker = format!("{}\x1b[1;31m{}\x1b[0m", " ".repeat($error.start.column + 6), "^".repeat($error.end.index - $error.start.index));

        eprintln!(
            "\x1b[1;36m{:?}\x1b[39m at {}:{}:{}\x1b[0m\n{}\n{}\n{}{}\n\n\x1b[1m{}\x1b[0m\n",
            $error.kind,
            $error.start.filename,
            $error.start.line,
            $error.start.column,
            line1,
            line2,
            marker,
            line3,
            $error.message,
        );
    };
}

macro_rules! exit {
    ($error:expr, $code:expr) => {{
        print_error!($error, $code);
        std::process::exit(1);
    }};
}

fn main() {
    let cli = Roost::from_args();

    let start_total = Instant::now();

    let mut code = String::new();
    let mut file = File::open(&cli.file).unwrap_or_else(|e| {
        eprintln!("\x1b[31mCould not read file \x1b[1m{}\x1b[22m: {}\x1b[0m", cli.file, e);
        std::process::exit(2);
    });
    file.read_to_string(&mut code).unwrap_or_else(|e| {
        eprintln!("\x1b[31mCould not read file \x1b[1m{}\x1b[22m: {}\x1b[0m", cli.file, e);
        std::process::exit(3);
    });

    let end_read = start_total.elapsed();
    let start = Instant::now();

    let mut lexer = Lexer::new(&code, cli.file);
    let tokens = match lexer.scan() {
        Ok(tokens) => tokens,
        Err(e) => exit!(e, code),
    };
    // file = File::create("tokens.txt").unwrap();
    // write!(file, "{:#?}", tokens).unwrap();

    let end_lex = start.elapsed();
    let start = Instant::now();

    let mut parser = Parser::new(&tokens);
    let nodes = match parser.parse() {
        Ok(nodes) => nodes,
        Err(errors) => {
            for error in errors {
                print_error!(error, code);
            }
            std::process::exit(1);
        },
    };
    // file = File::create("nodes.txt").unwrap();
    // write!(file, "{:#?}", nodes).unwrap();

    let end_parse = start.elapsed();
    let start = Instant::now();

    let mut interpreter = Interpreter::new(
        nodes,
        |m| print!("{}", m),
        |code| std::process::exit(code),
    );
    interpreter.run().unwrap_or_else(|e| exit!(e, code));

    let end_run = start.elapsed();
    let end = start_total.elapsed();
    if cli.time {
        println!(
            "\n\x1b[36m-----------------------\n{: <15} {:?}\n{: <15} {:?}\n{: <15} {:?}\n{: <15} {:?}\n{: <15} {:?}\x1b[0m",
            "Read File:",   end_read,
            "Scan tokens:", end_lex,
            "Parse AST:",   end_parse,
            "Run:",         end_run,
            "Total:",       end,
        );
    }
}
