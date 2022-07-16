use roost::{lexer::Lexer, parser::Parser, tokens::TokenType};

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
}
