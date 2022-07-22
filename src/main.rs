use std::{io, process};

use roost::{interpreter::Interpreter, lexer::Lexer, parser::Parser, tokens::TokenKind};

fn main() {
    let mut l = Lexer::new(r#"< > <= >= << >> <<= >>= . .. ..= .3"#);
    while let Ok(token) = l.next_token() {
        println!("{token:?}");
        if let TokenKind::Eof = token.kind {
            break;
        }
    }

    let l = Lexer::new(r#"var a = 1 ;"#);
    let mut p = Parser::new(l);
    println!("{:#?}", p.parse());

    let l = Lexer::new(
        r#"
        printl('asd')
        fun lel(a, b) {
            printl(a)
            a + b
        }
        var lal = fun(a, b) {
            printl(b)
            a - b
        }
        printl(lel(3, 5))
        printl(lal(3, 5))

        class Test {
            static var staticVar1
            static var staticVar2 = 42
            var attribute1
            var attribute2 = 84

            static fun staticFun() {
                printl(this.staticVar1, Test.staticVar2)
                try this.attribute1 catch (err) debug(err)
            }

            fun method() {
                printl(this.attribute1, this.attribute2)
                try this.staticVar1 catch (err) debug(err)
            }

            fun setter(new) {
                this.attribute1 = new
            }

            fun getter() {
                this.attribute1
            }
        }

        Test.staticFun()
        var test = Test()
        test.method()
        printl(test.attribute1, test.getter())
        test.setter(42)
        printl(test.attribute1, test.getter())

        try a catch (err) err
        "#,
    );
    let tree = Parser::new(l).parse().unwrap_or_else(|e| {
        eprintln!("{:?}", e);
        process::exit(1);
    });
    println!(
        "{:?}",
        Interpreter::new(&tree, io::stdout(), io::stderr(), |code| process::exit(
            code
        ))
        .run(true),
    );
}
