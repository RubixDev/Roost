use std::{io::{Read, /* Write */}, time::Instant, fs::File, collections::HashMap};
use rustyline::{Editor, error::ReadlineError};
use structopt::StructOpt;
use roost::{lexer::Lexer, parser::Parser, interpreter::{Interpreter, value::Value}};

/// Command line interpreter for the roost language
#[derive(StructOpt, Clone)]
#[structopt(author)]
struct Roost {
    /// File to run
    #[structopt()]
    file: Option<String>,

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
            "\x1b[1;36m{:?}\x1b[39m at {}:{}:{}\x1b[0m\n{}\n{}\n{}{}\n\n\x1b[1;31m{}\x1b[0m\n",
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
    match cli.file.clone() {
        Some(file) => run_file(cli, file),
        None => run_repl(),
    }
}

fn run_file(cli: Roost, filename: String) {
    let start_total = Instant::now();

    let mut code = String::new();
    let mut file = File::open(&filename).unwrap_or_else(|e| {
        eprintln!("\x1b[31mCould not read file \x1b[1m{}\x1b[22m: {}\x1b[0m", filename, e);
        std::process::exit(2);
    });
    file.read_to_string(&mut code).unwrap_or_else(|e| {
        eprintln!("\x1b[31mCould not read file \x1b[1m{}\x1b[22m: {}\x1b[0m", filename, e);
        std::process::exit(3);
    });

    let end_read = start_total.elapsed();
    let start = Instant::now();

    let nodes = match Parser::new_parse(Lexer::new(&code, filename)) {
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

    Interpreter::new_run(
        nodes,
        |m| print!("{}", m),
        |code| std::process::exit(code),
    ).unwrap_or_else(|e| exit!(e, code));

    let end_run = start.elapsed();
    let end = start_total.elapsed();
    if cli.time {
        println!(
            "\n\x1b[36m-----------------------\n{: <15} {:?}\n{: <15} {:?}\n{: <15} {:?}\n{: <15} {:?}\x1b[0m",
            "Read File:",   end_read,
            "Parse AST:",   end_parse,
            "Run:",         end_run,
            "Total:",       end,
        );
    }
}

fn run_repl() {
    let mut global_scope: HashMap<String, Value> = HashMap::new();
    let mut rl = Editor::<()>::new();
    loop {
        let line = rl.readline(">> ");
        match line {
            Ok(line) => {
                rl.add_history_entry(&line);
                if line.chars().all(|char| [' ', '\t', '\r'].contains(&char)) { continue; }

                let nodes = match Parser::new_parse(Lexer::new(&line, String::from("<stdin>"))) {
                    Ok(nodes) => nodes,
                    Err(errors) => {
                        for error in errors {
                            print_error!(error, line);
                        }
                        continue;
                    },
                };

                let mut interpreter = Interpreter::new(
                    nodes,
                    |m| print!("{}", m),
                    |code| std::process::exit(code),
                );
                interpreter.scopes.push(global_scope.clone());
                interpreter.current_scope_index += 1;
                match interpreter.run(false) {
                    Ok(result) => {
                        println!("\x1b[90m{}\x1b[0m", result.value.unwrap_or(Value::Null))
                    },
                    Err(error) => {
                        print_error!(error, line);
                        continue;
                    },
                }
                global_scope = interpreter.scopes[1].clone();
            },
            Err(ReadlineError::Eof) => break,
            Err(_) => std::process::exit(1),
        }
    }
}
