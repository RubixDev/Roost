#[macro_use]
mod error;
mod tokens;
mod nodes;
pub mod lexer;
pub mod parser;
pub mod interpreter;

#[cfg(test)]
mod tests {
    mod fetch;
    mod all;
}
