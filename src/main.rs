use std::{io, process};

use roost::{interpreter::Interpreter, lexer::Lexer, parser::Parser, tokens::TokenType};

fn main() {
    let mut l = Lexer::new(r#"< > <= >= << >> <<= >>= . .. ..= .3"#);
    while let Ok(token) = l.next_token() {
        println!("{token:?}");
        if let TokenType::Eof = token.token_type {
            break;
        }
    }

    let l = Lexer::new(r#"var a = 1 ;"#);
    let mut p = Parser::new(l);
    println!("{:#?}", p.parse());

    let l = Lexer::new(
        r#"
        println('asd');
        fun lel(a, b) {
            println(a);
            a + b
        };
        var lal = fun(a, b) {
            println(b);
            a - b
        };
        println(lel(3, 5));
        println(lal(3, 5));

        class Test {
            static var staticVar1;
            static var staticVar2 = 42;
            var attribute1;
            var attribute2 = 84;

            static fun staticFun() {
                println(this.staticVar1, Test.staticVar2);
                try this.attribute1 catch (err) println(err);
            };

            fun method() {
                println(this.attribute1, this.attribute2);
                try this.staticVar1 catch (err) println(err);
            };

            fun setter(new) {
                this.attribute1 = new;
            };

            fun getter() {
                this.attribute1
            };
        };

        Test.staticFun();
        var test = Test();
        test.method();
        println(test.attribute1, test.getter());
        test.setter(42);
        println(test.attribute1, test.getter());

        try a catch (err) err
        "#,
    );
    let tree = Parser::new(l).parse().unwrap_or_else(|e| {
        eprintln!("{:?}", e);
        process::exit(1);
    });
    println!(
        "{:?}",
        Interpreter::new(&tree, io::stdout(), |code| process::exit(code)).run(true),
    );
}
