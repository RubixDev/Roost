//mod repl_helper;

//use repl_helper::ReplHelper;
use roost::{interpreter::Interpreter, lexer::Lexer, parser::Parser};
//use rustyline::{error::ReadlineError, Config, Editor};
use std::{
    fs::File,
    io::{self, Read},
    process,
    time::Instant,
};

#[cfg(test)]
mod tests {
    mod all;
    mod fetch;
}

/// Command line interpreter for the roost language
#[derive(clap::Parser, Clone)]
#[clap(author, version, about)]
struct Roost {
    /// File to run
    file: Option<String>,

    /// Measure and display the time of execution
    #[clap(short, long)]
    time: bool,
}

macro_rules! print_error {
    ($error:expr, $filename:expr, $code:expr $(,)?) => {
        let lines: Vec<&str> = $code.split('\n').collect();

        let line1 = if $error.span.start.line > 1 {
            format!(
                "\n \x1b[90m{: >3} | \x1b[0m{}",
                $error.span.start.line - 1,
                lines[$error.span.start.line - 2]
            )
        } else {
            String::new()
        };
        let line2 = format!(
            " \x1b[90m{: >3} | \x1b[0m{}",
            $error.span.start.line,
            lines[$error.span.start.line - 1]
        );
        let line3 = if $error.span.start.line < lines.len() {
            format!(
                "\n \x1b[90m{: >3} | \x1b[0m{}",
                $error.span.start.line + 1,
                lines[$error.span.start.line]
            )
        } else {
            String::new()
        };

        let marker = format!(
            "{}\x1b[1;31m{}\x1b[0m",
            " ".repeat($error.span.start.column + 6),
            if $error.span.start.line == $error.span.end.line
                || $error.span.start.index == $error.span.end.index - 1
            {
                "^".repeat($error.span.end.index - $error.span.start.index)
            } else {
                "^".repeat(lines[$error.span.start.line - 1].len() - $error.span.start.column + 1)
                    + "..."
            },
        );

        eprintln!(
            "\x1b[1;36m{:?}\x1b[39m at {}:{}:{}\x1b[0m\n{}\n{}\n{}{}\n\n\x1b[1;31m{}\x1b[0m\n",
            $error.kind,
            $filename,
            $error.span.start.line,
            $error.span.start.column,
            line1,
            line2,
            marker,
            line3,
            $error.message,
        );
    };
}

macro_rules! exit {
    ($($arg:tt)*) => {{
        print_error!($($arg)*);
        process::exit(1);
    }};
}

fn main() {
    use clap::Parser;
    let cli = Roost::parse();
    match cli.file {
        Some(_) => run_file(cli),
        None => todo!(),
    }
}

fn run_file(cli: Roost) {
    let filename = &cli.file.unwrap();
    let start_total = Instant::now();

    let mut code = String::new();
    let mut file = File::open(filename).unwrap_or_else(|e| {
        eprintln!(
            "\x1b[31mCould not read file \x1b[1m{}\x1b[22m: {}\x1b[0m",
            filename, e,
        );
        process::exit(2);
    });
    file.read_to_string(&mut code).unwrap_or_else(|e| {
        eprintln!(
            "\x1b[31mCould not read file \x1b[1m{}\x1b[22m: {}\x1b[0m",
            filename, e,
        );
        process::exit(2);
    });

    let end_read = start_total.elapsed();
    let start = Instant::now();

    let nodes = match Parser::new(Lexer::new(&code)).parse() {
        Ok(nodes) => nodes,
        Err(errors) => {
            for error in errors {
                print_error!(error, filename, code);
            }
            process::exit(1);
        }
    };

    let end_parse = start.elapsed();
    let start = Instant::now();

    Interpreter::new(&nodes, io::stdout(), io::stderr(), |code| {
        let end_run = start.elapsed();
        let end = start_total.elapsed();
        if cli.time {
            println!(
                "\n\x1b[36m-----------------------\n{: <15} {:?}\n{: <15} {:?}\n{: <15} {:?}\n{: <15} {:?}\x1b[0m",
                "Read File:", end_read,
                "Parse AST:", end_parse,
                "Run:",       end_run,
                "Total:",     end,
            );
        }
        process::exit(code);
    }).run(true)
    .unwrap_or_else(|e| exit!(e, filename, code));

    let end_run = start.elapsed();
    let end = start_total.elapsed();
    if cli.time {
        println!(
            "\n\x1b[36m-----------------------\n{: <15} {:?}\n{: <15} {:?}\n{: <15} {:?}\n{: <15} {:?}\x1b[0m",
            "Read File:", end_read,
            "Parse AST:", end_parse,
            "Run:",       end_run,
            "Total:",     end,
        );
    }
}

// TODO: save filename in Location for correct errors
//macro_rules! repl_num {
//    ($error:ident) => {{
//        let num = $error.start.filename.split("repl-").collect::<Vec<&str>>()[1];
//        let num = &num[..num.len() - 1];
//        num.parse::<usize>().unwrap()
//    }};
//}

// TODO: REPL
/*
fn run_repl() {
    let mut trees = vec![];
    let trees_ptr = &mut trees as *mut Vec<Statements>;
    let mut global_scope = HashMap::new();
    let mut inputs = vec![];
    let mut rl = Editor::with_config(
        Config::builder()
            .completion_type(rustyline::CompletionType::List)
            .tab_stop(4)
            .indent_size(4)
            .build(),
    )
    .unwrap();
    rl.set_helper(Some(ReplHelper::new(global_scope.clone())));
    match std::env::var("HOME") {
        Ok(path) => {
            let _ = rl.load_history(&format!("{}/.roost_history", path));
        }
        Err(_) => {}
    }
    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                rl.add_history_entry(&line);
                inputs.push(line.clone());
                if line.chars().all(|char| [' ', '\t', '\r'].contains(&char)) {
                    continue;
                }

                let nodes = match Parser::new(Lexer::new(&line)).parse() {
                    Ok(nodes) => nodes,
                    Err(errors) => {
                        for error in errors {
                            print_error!(
                                error,
                                // TODO: save filename in Location for correct errors
                                format!("<repl-{}>", inputs.len()),
                                inputs[inputs.len() - 1],
                            );
                        }
                        continue;
                    }
                };
                unsafe { (*trees_ptr).push(nodes) }
                let nodes = trees.last().unwrap();

                let mut interpreter =
                    Interpreter::new(&nodes, io::stdout(), io::stderr(), |code| {
                        if let Ok(path) = env::var("XDG_STATE_HOME") {
                            let _ = rl.save_history(&Path::new(&path).join("roost/history"));
                        } else if let Ok(path) = env::var("HOME") {
                            let _ = rl.save_history(&Path::new(&path).join(".local/state/roost/history"));
                        }
                        process::exit(code);
                    });
                interpreter.scopes.push(global_scope.clone());
                interpreter.scope_idx += 1;
                match interpreter.run(false) {
                    Ok(result) => {
                        if let Some(val) = result.value {
                            println!("{:?}", val.borrow());
                        }
                    }
                    Err(error) => {
                        print_error!(
                            error,
                            // TODO: save filename in Location for correct errors
                            format!("<repl-{}>", inputs.len()),
                            inputs[inputs.len() - 1]
                        );
                        continue;
                    }
                }
                mem::swap(&mut global_scope, &mut interpreter.scopes[1]);
                rl.set_helper(Some(ReplHelper::new(global_scope.clone())));
            }
            Err(ReadlineError::Eof) => break,
            Err(ReadlineError::Interrupted) => continue,
            Err(_) => std::process::exit(1),
        }
    }
    match std::env::var("HOME") {
        Ok(path) => {
            let _ = rl.save_history(&format!("{}/.roost_history", path));
        }
        Err(_) => {}
    }
}
*/
