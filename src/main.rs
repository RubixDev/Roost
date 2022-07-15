use roost::{lexer::Lexer, tokens::TokenType, parser::Parser};

fn main() {
    let mut l = Lexer::new(r#"< > <= >= << >> <<= >>= . .. ..= .3"#, "asd");
    while let Ok(token) = l.next_token() {
        println!("{token:?}");
        if let TokenType::Eof = token.token_type {
            break;
        }
    }

    let l = Lexer::new(r#"var a = 1;"#, "asd");
    let mut p = Parser::new(l);
    println!("{:#?}", p.parse());
}
