use core::panic;
use bigdecimal::BigDecimal;
use num_bigint::BigInt;

use crate::{
    tokens::{Token, TokenType},
    nodes::{
        Statements,
        Statement,
        DeclareStatement,
        AssignStatement,
        AssignOperator,
        IfStatement,
        LoopStatement,
        WhileStatement,
        ForStatement,
        FunctionDeclaration,
        ReturnStatement,
        Expression,
        OrExpression,
        AndExpression,
        EqualityExpression,
        EqualityOperator,
        RelationalExpression,
        RelationalOperator,
        AdditiveExpression,
        AdditiveOperator,
        MultiplicativeExpression,
        MultiplicativeOperator,
        UnaryExpression,
        UnaryOperator,
        Atom,
        Number,
        TernaryExpression,
        CallExpression,
    },
};

pub struct Parser {
    tokens: Vec<Token>,
    current_token: Token,
    current_token_index: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        let first_token = tokens[0].clone();
        return Parser {
            tokens,
            current_token: first_token,
            current_token_index: 0,
        };
    }

    pub fn parse(&mut self) -> Statements {
        let statements = self.statements();
        if self.current_token.token_type != TokenType::EOF {
            panic!("SyntaxError at position {}: Expected EOF", self.current_token.position)
        }
        return statements;
    }

    fn update_current_token(&mut self) {
        self.current_token = self.tokens
            .get(self.current_token_index)
            .unwrap_or(&Token::new(TokenType::EOF, "", self.current_token.position))
            .clone();
    }

    fn advance(&mut self) {
        self.current_token_index += 1;
        self.update_current_token();
    }

    fn next_token(&self) -> Token {
        return self.tokens
            .get(self.current_token_index + 1)
            .unwrap_or(&Token::new(TokenType::EOF, "", self.current_token.position))
            .clone();
    }

    fn panic_expected(&self, token_type: TokenType, name: &str) {
        if self.current_token.token_type != token_type {
            panic!(
                "SyntaxError at position {}: Expected {}, found '{}'",
                self.current_token.position,
                name,
                self.current_token.value,
            )
        }
    }

    // ---------------------------------------

    fn statements(&mut self) -> Statements {
        while self.current_token.token_type == TokenType::EOL {
            self.advance()
        }

        if ![TokenType::EOF, TokenType::RBrace].contains(&self.current_token.token_type) {
            let mut statements: Vec<Statement> = vec![];
            statements.push(self.statement());
            loop {
                if self.current_token.token_type != TokenType::EOL {
                    panic!("SyntaxError at position {}: ';' or line break expected, found '{}'", self.current_token.position, self.current_token.value)
                }
                while self.current_token.token_type == TokenType::EOL {
                    self.advance();
                }
                if [TokenType::EOF, TokenType::RBrace].contains(&self.current_token.token_type) {
                    break;
                }
                statements.push(self.statement());
            }
            return Statements { statements };
        }

        return Statements { statements: vec![] };
    }

    fn block(&mut self) -> Statements {
        let statements: Vec<Statement>;
        if self.current_token.token_type == TokenType::LBrace {
            self.advance();

            statements = self.statements().statements;

            self.panic_expected(TokenType::RBrace, "'}'");
            self.advance();
        } else {
            statements = vec![self.statement()];
        }

        return Statements { statements };
    }

    fn statement(&mut self) -> Statement {
        if self.current_token.matches(TokenType::Keyword, "var") {
            return Statement::Declare(self.declare_statement());
        } else if self.current_token.matches(TokenType::Keyword, "if") {
            return Statement::If(self.if_statement());
        } else if self.current_token.matches(TokenType::Keyword, "loop") {
            return Statement::Loop(self.loop_statement());
        } else if self.current_token.matches(TokenType::Keyword, "while") {
            return Statement::While(self.while_statement());
        } else if self.current_token.matches(TokenType::Keyword, "for") {
            return Statement::For(self.for_statement());
        } else if self.current_token.matches(TokenType::Keyword, "fun") {
            return Statement::Function(self.function_declaration());
        } else if self.current_token.matches(TokenType::Keyword, "break") {
            self.advance();
            return Statement::Break;
        } else if self.current_token.matches(TokenType::Keyword, "continue") {
            self.advance();
            return Statement::Continue;
        } else if self.current_token.matches(TokenType::Keyword, "return") {
            return Statement::Return(self.return_statement());
        } else if self.current_token.token_type == TokenType::Identifier
            && [
                TokenType::Assign,
                TokenType::PlusAssign,
                TokenType::MinusAssign,
                TokenType::MultiplyAssign,
                TokenType::DivideAssign,
            ].contains(&self.next_token().token_type) {
            return Statement::Assign(self.assign_statement());
        } else {
            return Statement::Expression(self.expression());
        }
    }

    fn declare_statement(&mut self) -> DeclareStatement {
        self.advance();

        self.panic_expected(TokenType::Identifier, "identifier");
        let identifier = self.current_token.value.clone();
        self.advance();

        self.panic_expected(TokenType::Assign, "'='");
        self.advance();

        let expression = self.expression();

        return DeclareStatement { identifier, expression } ;
    }

    fn assign_statement(&mut self) -> AssignStatement {
        let identifier = self.current_token.value.clone();
        self.advance();

        let operator = match self.current_token.token_type {
            TokenType::Assign         => AssignOperator::Normal,
            TokenType::PlusAssign     => AssignOperator::Plus,
            TokenType::MinusAssign    => AssignOperator::Minus,
            TokenType::MultiplyAssign => AssignOperator::Multiply,
            TokenType::DivideAssign   => AssignOperator::Divide,
            _ => panic!(),
        };
        self.advance();

        let expression = self.expression();

        return AssignStatement { identifier, operator, expression } ;
    }

    fn if_statement(&mut self) -> IfStatement {
        self.advance();

        self.panic_expected(TokenType::LParen, "'('");
        self.advance();

        let condition = self.expression();

        self.panic_expected(TokenType::RParen, "')'");
        self.advance();

        let block = self.block();
        let else_block: Statements;

        if self.current_token.matches(TokenType::Keyword, "else") {
            self.advance();

            else_block = self.block();
        } else {
            else_block = Statements { statements: vec![] };
        }

        return IfStatement { condition, block, else_block } ;
    }

    fn loop_statement(&mut self) -> LoopStatement {
        self.advance();

        let block = self.block();

        return LoopStatement { block } ;
    }

    fn while_statement(&mut self) -> WhileStatement {
        self.advance();

        self.panic_expected(TokenType::LParen, "'('");
        self.advance();

        let condition = self.expression();

        self.panic_expected(TokenType::RParen, "')'");
        self.advance();

        let block = self.block();

        return WhileStatement { condition, block } ;
    }

    fn for_statement(&mut self) -> ForStatement {
        self.advance();

        self.panic_expected(TokenType::LParen, "'('");
        self.advance();

        self.panic_expected(TokenType::Identifier, "identifier");
        let identifier = self.current_token.value.clone();
        self.advance();

        if !self.current_token.matches(TokenType::Keyword, "in") {
            self.panic_expected(TokenType::EOF, "'in'");
        }
        self.advance();

        let expression = self.expression();

        self.panic_expected(TokenType::RParen, "')'");
        self.advance();

        let block = self.block();

        return ForStatement { identifier, expression, block } ;
    }

    fn function_declaration(&mut self) -> FunctionDeclaration {
        self.advance();

        self.panic_expected(TokenType::Identifier, "identifier");
        let identifier = self.current_token.value.clone();
        self.advance();

        self.panic_expected(TokenType::LParen, "'('");
        self.advance();

        let mut params: Vec<String> = vec![];
        if self.current_token.token_type != TokenType::RParen {
            self.panic_expected(TokenType::Identifier, "identifier");
            params.push(self.current_token.value.clone());
            self.advance();

            while self.current_token.token_type == TokenType::Comma {
                self.advance();

                self.panic_expected(TokenType::Identifier, "identifier");
                params.push(self.current_token.value.clone());
                self.advance();
            }
        }

        self.panic_expected(TokenType::RParen, "')'");
        self.advance();

        let block = self.block();

        return FunctionDeclaration { identifier, params, block } ;
    }

    fn return_statement(&mut self) -> ReturnStatement {
        self.advance();

        let mut expression = None;
        if ![TokenType::EOL, TokenType::EOF].contains(&self.current_token.token_type) {
            expression = Some(self.expression());
        }

        return ReturnStatement { expression } ;
    }


    fn expression(&mut self) -> Expression {
        let base = self.ternary_expression();

        let mut range = None;
        if self.current_token.token_type == TokenType::RangeDots {
            let inclusive = self.current_token.value == "..=";
            self.advance();

            let upper = self.ternary_expression();

            range = Some((inclusive, upper));
        }

        return Expression { base: Box::new(base), range: Box::new(range) };
    }

    fn ternary_expression(&mut self) -> TernaryExpression {
        let base = self.or_expression();

        let mut ternary = None;
        if self.current_token.token_type == TokenType::QuestionMark {
            self.advance();

            let ternary_if = self.expression();

            self.panic_expected(TokenType::Colon, "':'");
            self.advance();

            let ternary_else = self.expression();
            ternary = Some((ternary_if, ternary_else));
        }

        return TernaryExpression { base, ternary };
    }

    fn or_expression(&mut self) -> OrExpression {
        let base = self.and_expression();

        let mut following = vec![];
        while self.current_token.token_type == TokenType::Or {
            self.advance();

            following.push(self.and_expression());
        }

        return OrExpression { base, following };
    }

    fn and_expression(&mut self) -> AndExpression {
        let base = self.equality_expression();

        let mut following = vec![];
        while self.current_token.token_type == TokenType::And {
            self.advance();

            following.push(self.equality_expression());
        }

        return AndExpression { base, following };
    }

    fn equality_expression(&mut self) -> EqualityExpression {
        let base = self.relational_expression();

        let mut following = vec![];
        while [
            TokenType::Equal,
            TokenType::NotEqual,
        ].contains(&self.current_token.token_type) {
            let operator = match self.current_token.token_type {
                TokenType::Equal    => EqualityOperator::Equal,
                TokenType::NotEqual => EqualityOperator::NotEqual,
                _ => panic!(),
            };
            self.advance();

            following.push((operator, self.relational_expression()));
        }

        return EqualityExpression { base, following };
    }

    fn relational_expression(&mut self) -> RelationalExpression {
        let base = self.additive_expression();

        let mut following = vec![];
        while [
            TokenType::LessThan,
            TokenType::GreaterThan,
            TokenType::LessThanOrEqual,
            TokenType::GreaterThanOrEqual,
        ].contains(&self.current_token.token_type) {
            let operator = match self.current_token.token_type {
                TokenType::LessThan           => RelationalOperator::LessThan,
                TokenType::GreaterThan        => RelationalOperator::GreaterThan,
                TokenType::LessThanOrEqual    => RelationalOperator::LessThanOrEqual,
                TokenType::GreaterThanOrEqual => RelationalOperator::GreaterThanOrEqual,
                _ => panic!(),
            };
            self.advance();

            following.push((operator, self.additive_expression()));
        }

        return RelationalExpression { base, following };
    }

    fn additive_expression(&mut self) -> AdditiveExpression {
        let base = self.multiplicative_expression();

        let mut following = vec![];
        while [
            TokenType::Plus,
            TokenType::Minus,
        ].contains(&self.current_token.token_type) {
            let operator = match self.current_token.token_type {
                TokenType::Plus  => AdditiveOperator::Plus,
                TokenType::Minus => AdditiveOperator::Minus,
                _ => panic!(),
            };
            self.advance();

            following.push((operator, self.multiplicative_expression()));
        }

        return AdditiveExpression { base, following };
    }

    fn multiplicative_expression(&mut self) -> MultiplicativeExpression {
        let base = self.unary_expression();

        let mut following = vec![];
        while [
            TokenType::Multiply,
            TokenType::Divide,
        ].contains(&self.current_token.token_type) {
            let operator = match self.current_token.token_type {
                TokenType::Multiply => MultiplicativeOperator::Multiply,
                TokenType::Divide   => MultiplicativeOperator::Divide,
                _ => panic!(),
            };
            self.advance();

            following.push((operator, self.unary_expression()));
        }

        return MultiplicativeExpression { base, following };
    }

    fn unary_expression(&mut self) -> UnaryExpression {
        if [
            TokenType::Plus,
            TokenType::Minus,
            TokenType::Not,
        ].contains(&self.current_token.token_type) {
            let operator = match self.current_token.token_type {
                TokenType::Plus  => UnaryOperator::Plus,
                TokenType::Minus => UnaryOperator::Minus,
                TokenType::Not   => UnaryOperator::Not,
                _ => panic!(),
            };
            return UnaryExpression::Operator(operator, Box::new(self.unary_expression()));
        }

        if self.current_token.token_type == TokenType::LParen {
            self.advance();

            let expression = self.expression();

            self.panic_expected(TokenType::RParen, "')'");
            self.advance();

            return UnaryExpression::Expression(expression);
        }

        return UnaryExpression::Atom(self.atom());
    }

    fn atom(&mut self) -> Atom {
        if self.current_token.matches(TokenType::Keyword, "null") {
            self.advance();

            return Atom::Null;
        }

        if self.current_token.token_type == TokenType::Number {
            let value = self.current_token.value.clone();
            let number = if value.contains('.') {
                Number::Float(value.parse::<BigDecimal>().unwrap())
            } else {
                Number::Int(value.parse::<BigInt>().unwrap())
            };
            self.advance();
            return Atom::Number(number);
        }

        if self.current_token.matches(TokenType::Keyword, "true")
            || self.current_token.matches(TokenType::Keyword, "false") {
            let value = self.current_token.value == "true";
            self.advance();

            return Atom::Bool(value);
        }

        if self.current_token.token_type == TokenType::String {
            let value = self.current_token.value.clone();
            self.advance();

            return Atom::String(value);
        }

        if self.current_token.token_type == TokenType::Identifier {
            let value = self.current_token.value.clone();

            if self.next_token().token_type == TokenType::LParen {
                return Atom::Call(self.call_expression());
            } else {
                self.advance();
            }

            return Atom::Identifier(value);
        }

        panic!("SyntaxError at position {}: Expected expression, found '{}'", self.current_token.position, self.current_token.value);
    }

    fn call_expression(&mut self) -> CallExpression {
        let identifier = self.current_token.value.clone();
        self.advance();

        let args = self.arguments();

        let mut following = vec![];
        while self.current_token.token_type == TokenType::Dot {
            self.advance();

            self.panic_expected(TokenType::Identifier, "identifier");
            let identifier = self.current_token.value.clone();
            self.advance();

            let args = self.arguments();

            following.push((identifier, args));
        }

        return CallExpression { identifier, args, following };
    }

    fn arguments(&mut self) -> Vec<Expression> {
        self.advance();

        let mut args = vec![];
        if self.current_token.token_type != TokenType::RParen {
            args.push(self.expression());

            while self.current_token.token_type == TokenType::Comma {
                self.advance();

                args.push(self.expression());
            }
        }

        self.panic_expected(TokenType::RParen, "')'");
        self.advance();

        return args;
    }
}
