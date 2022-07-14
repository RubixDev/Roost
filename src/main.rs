use roost::{lexer::Lexer, tokens::TokenType};

fn main() {
    let mut l = Lexer::new(r#"< > <= >= << >> <<= >>= . .. ..= .3"#, "asd");
    while let Ok(token) = l.next_token() {
        println!("{token:?}");
        if let TokenType::Eof = token.token_type {
            break;
        }
    }
}
