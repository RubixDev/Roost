use core::panic;
use std::slice::Iter;
use rust_decimal::{Decimal, Error};
use crate::{
    tokens::{Token, TokenType},
    error::{Result, Location},
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
        TernaryExpression,
        CallExpression,
        ExponentialExpression,
    },
};

macro_rules! syntax {
    ($self:ident, $($arg:tt)*) => {
        error!(SyntaxError, $self.current_token.location.clone(), $($arg)*)
    };
}

macro_rules! expected {
    ($self:ident, $token_type:ident, $name:expr) => {
        if $self.current_token.token_type != TokenType::$token_type {
            error!(SyntaxError, $self.current_token.location.clone(), "Expected {}, found '{}'", $name, $self.current_token.value);
        }
    };
}

pub struct Parser<'a> {
    tokens: Iter<'a, Token>,
    current_token: Token,
}

impl <'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        return Parser {
            tokens: tokens.iter(),
            current_token: Token::dummy(),
        };
    }

    pub fn parse(&mut self) -> Result<Statements> {
        self.advance();
        let statements = self.statements()?;
        if self.current_token.token_type != TokenType::EOF {
            syntax!(self, "Expected EOF");
        }
        return Ok(statements);
    }

    fn advance(&mut self) {
        self.current_token = self.tokens
            .next()
            .unwrap_or(&Token::dummy())
            .clone();
    }

    fn next_token(&self) -> Token {
        return self.tokens
            .clone()
            .next()
            .unwrap_or(&Token::dummy())
            .clone();
    }

    // ---------------------------------------

    fn statements(&mut self) -> Result<Statements> {
        let start_location = self.current_token.location.clone();
        while self.current_token.token_type == TokenType::EOL {
            self.advance()
        }

        if ![TokenType::EOF, TokenType::RBrace].contains(&self.current_token.token_type) {
            let mut statements: Vec<Statement> = vec![];
            statements.push(self.statement()?);
            loop {
                if [TokenType::EOF, TokenType::RBrace].contains(&self.current_token.token_type) { break; }
                if self.current_token.token_type != TokenType::EOL {
                    syntax!(self, "Expected ';' or line break, found '{}'", self.current_token.value);
                }
                while self.current_token.token_type == TokenType::EOL {
                    self.advance();
                }
                if [TokenType::EOF, TokenType::RBrace].contains(&self.current_token.token_type) { break; }
                statements.push(self.statement()?);
            }
            return Ok(Statements { location: start_location, statements });
        }

        return Ok(Statements { location: start_location, statements: vec![] });
    }

    fn block(&mut self) -> Result<Statements> {
        let start_location = self.current_token.location.clone();
        while self.current_token.token_type == TokenType::EOL {
            self.advance();
        }

        let statements: Vec<Statement>;
        if self.current_token.token_type == TokenType::LBrace {
            self.advance();

            statements = self.statements()?.statements;

            expected!(self, RBrace, "'}'");
            self.advance();
        } else {
            statements = vec![self.statement()?];
        }

        return Ok(Statements { location: start_location, statements });
    }

    fn statement(&mut self) -> Result<Statement> {
        if self.current_token.matches(TokenType::Keyword, "var") {
            return Ok(Statement::Declare(self.declare_statement()?));
        } else if self.current_token.matches(TokenType::Keyword, "if") {
            return Ok(Statement::If(self.if_statement()?));
        } else if self.current_token.matches(TokenType::Keyword, "loop") {
            return Ok(Statement::Loop(self.loop_statement()?));
        } else if self.current_token.matches(TokenType::Keyword, "while") {
            return Ok(Statement::While(self.while_statement()?));
        } else if self.current_token.matches(TokenType::Keyword, "for") {
            return Ok(Statement::For(self.for_statement()?));
        } else if self.current_token.matches(TokenType::Keyword, "fun") {
            return Ok(Statement::Function(self.function_declaration()?));
        } else if self.current_token.matches(TokenType::Keyword, "break") {
            self.advance();
            return Ok(Statement::Break);
        } else if self.current_token.matches(TokenType::Keyword, "continue") {
            self.advance();
            return Ok(Statement::Continue);
        } else if self.current_token.matches(TokenType::Keyword, "return") {
            return Ok(Statement::Return(self.return_statement()?));
        } else if self.current_token.token_type == TokenType::Identifier
            && [
                TokenType::Assign,
                TokenType::PlusAssign,
                TokenType::MinusAssign,
                TokenType::MultiplyAssign,
                TokenType::DivideAssign,
                TokenType::ModuloAssign,
                TokenType::IntDivideAssign,
            ].contains(&self.next_token().token_type) {
            return Ok(Statement::Assign(self.assign_statement()?));
        } else {
            return Ok(Statement::Expression(self.expression()?));
        }
    }

    fn declare_statement(&mut self) -> Result<DeclareStatement> {
        let start_location = self.current_token.location.clone();
        self.advance();

        expected!(self, Identifier, "identifier");
        let identifier = self.current_token.value.clone();
        self.advance();

        expected!(self, Assign, "'='");
        self.advance();

        let expression = self.expression()?;

        return Ok(DeclareStatement { location: start_location, identifier, expression }) ;
    }

    fn assign_statement(&mut self) -> Result<AssignStatement> {
        let start_location = self.current_token.location.clone();
        let identifier = self.current_token.value.clone();
        self.advance();

        let operator = match self.current_token.token_type {
            TokenType::Assign          => AssignOperator::Normal,
            TokenType::PlusAssign      => AssignOperator::Plus,
            TokenType::MinusAssign     => AssignOperator::Minus,
            TokenType::MultiplyAssign  => AssignOperator::Multiply,
            TokenType::DivideAssign    => AssignOperator::Divide,
            TokenType::ModuloAssign    => AssignOperator::Modulo,
            TokenType::IntDivideAssign => AssignOperator::IntDivide,
            _ => panic!(),
        };
        self.advance();

        let expression = self.expression()?;

        return Ok(AssignStatement { location: start_location, identifier, operator, expression });
    }

    fn if_statement(&mut self) -> Result<IfStatement> {
        let start_location = self.current_token.location.clone();
        self.advance();

        expected!(self, LParen, "'('");
        self.advance();

        let condition = self.expression()?;

        expected!(self, RParen, "')'");
        self.advance();

        let block = self.block()?;
        let else_block: Statements;

        let mut tokens = self.tokens.clone();
        let mut current_token = self.current_token.clone();
        let mut count = 0;

        while current_token.token_type == TokenType::EOL {
            current_token = tokens.next().unwrap_or(&Token::dummy()).clone();
            count += 1;
        }

        if current_token.matches(TokenType::Keyword, "else") {
            for _ in 0..count { self.advance(); }
            self.advance();

            else_block = self.block()?;
        } else {
            else_block = Statements { location: Location::new(start_location.filename.clone()), statements: vec![] };
        }

        return Ok(IfStatement { location: start_location, condition, block, else_block });
    }

    fn loop_statement(&mut self) -> Result<LoopStatement> {
        let start_location = self.current_token.location.clone();
        self.advance();

        let block = self.block()?;

        return Ok(LoopStatement { location: start_location, block });
    }

    fn while_statement(&mut self) -> Result<WhileStatement> {
        let start_location = self.current_token.location.clone();
        self.advance();

        expected!(self, LParen, "'('");
        self.advance();

        let condition = self.expression()?;

        expected!(self, RParen, "')'");
        self.advance();

        let block = self.block()?;

        return Ok(WhileStatement { location: start_location, condition, block });
    }

    fn for_statement(&mut self) -> Result<ForStatement> {
        let start_location = self.current_token.location.clone();
        self.advance();

        expected!(self, LParen, "'('");
        self.advance();

        expected!(self, Identifier, "identifier");
        let identifier = self.current_token.value.clone();
        self.advance();

        if !self.current_token.matches(TokenType::Keyword, "in") {
            expected!(self, EOF, "'in'");
        }
        self.advance();

        let expression = self.expression()?;

        expected!(self, RParen, "')'");
        self.advance();

        let block = self.block()?;

        return Ok(ForStatement { location: start_location, identifier, expression, block });
    }

    fn function_declaration(&mut self) -> Result<FunctionDeclaration> {
        let start_location = self.current_token.location.clone();
        self.advance();

        expected!(self, Identifier, "identifier");
        let identifier = self.current_token.value.clone();
        self.advance();

        expected!(self, LParen, "'('");
        self.advance();

        let mut params: Vec<String> = vec![];
        if self.current_token.token_type != TokenType::RParen {
            expected!(self, Identifier, "identifier");
            params.push(self.current_token.value.clone());
            self.advance();

            while self.current_token.token_type == TokenType::Comma {
                self.advance();

                expected!(self, Identifier, "identifier");
                params.push(self.current_token.value.clone());
                self.advance();
            }
        }

        expected!(self, RParen, "')'");
        self.advance();

        let block = self.block()?;

        return Ok(FunctionDeclaration { location: start_location, identifier, params, block });
    }

    fn return_statement(&mut self) -> Result<ReturnStatement> {
        let start_location = self.current_token.location.clone();
        self.advance();

        let mut expression = None;
        if ![TokenType::EOL, TokenType::EOF].contains(&self.current_token.token_type) {
            expression = Some(self.expression()?);
        }

        return Ok(ReturnStatement { location: start_location, expression });
    }


    fn expression(&mut self) -> Result<Expression> {
        let start_location = self.current_token.location.clone();
        let base = self.ternary_expression()?;

        let mut range = None;
        if self.current_token.token_type == TokenType::RangeDots {
            let inclusive = self.current_token.value == "..=";
            self.advance();

            let upper = self.ternary_expression()?;

            range = Some((inclusive, upper));
        }

        return Ok(Expression { location: start_location, base: Box::new(base), range: Box::new(range) });
    }

    fn ternary_expression(&mut self) -> Result<TernaryExpression> {
        let start_location = self.current_token.location.clone();
        let base = self.or_expression()?;

        let mut ternary = None;
        if self.current_token.token_type == TokenType::QuestionMark {
            self.advance();

            let ternary_if = self.expression()?;

            expected!(self, Colon, "':'");
            self.advance();

            let ternary_else = self.expression()?;
            ternary = Some((ternary_if, ternary_else));
        }

        return Ok(TernaryExpression { location: start_location, base, ternary });
    }

    fn or_expression(&mut self) -> Result<OrExpression> {
        let start_location = self.current_token.location.clone();
        let base = self.and_expression()?;

        let mut following = vec![];
        while self.current_token.token_type == TokenType::Or {
            self.advance();

            following.push(self.and_expression()?);
        }

        return Ok(OrExpression { location: start_location, base, following });
    }

    fn and_expression(&mut self) -> Result<AndExpression> {
        let start_location = self.current_token.location.clone();
        let base = self.equality_expression()?;

        let mut following = vec![];
        while self.current_token.token_type == TokenType::And {
            self.advance();

            following.push(self.equality_expression()?);
        }

        return Ok(AndExpression { location: start_location, base, following });
    }

    fn equality_expression(&mut self) -> Result<EqualityExpression> {
        let start_location = self.current_token.location.clone();
        let base = self.relational_expression()?;

        let mut other = None;
        if [
            TokenType::Equal,
            TokenType::NotEqual,
        ].contains(&self.current_token.token_type) {
            let operator = match self.current_token.token_type {
                TokenType::Equal    => EqualityOperator::Equal,
                TokenType::NotEqual => EqualityOperator::NotEqual,
                _ => panic!(),
            };
            self.advance();

            other = Some((operator, self.relational_expression()?));
        }

        return Ok(EqualityExpression { location: start_location, base, other });
    }

    fn relational_expression(&mut self) -> Result<RelationalExpression> {
        let start_location = self.current_token.location.clone();
        let base = self.additive_expression()?;

        let mut other = None;
        if [
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

            other = Some((operator, self.additive_expression()?));
        }

        return Ok(RelationalExpression { location: start_location, base, other });
    }

    fn additive_expression(&mut self) -> Result<AdditiveExpression> {
        let start_location = self.current_token.location.clone();
        let base = self.multiplicative_expression()?;

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

            following.push((operator, self.multiplicative_expression()?));
        }

        return Ok(AdditiveExpression { location: start_location, base, following });
    }

    fn multiplicative_expression(&mut self) -> Result<MultiplicativeExpression> {
        let start_location = self.current_token.location.clone();
        let base = self.unary_expression()?;

        let mut following = vec![];
        while [
            TokenType::Multiply,
            TokenType::Divide,
            TokenType::Modulo,
            TokenType::IntDivide,
        ].contains(&self.current_token.token_type) {
            let operator = match self.current_token.token_type {
                TokenType::Multiply  => MultiplicativeOperator::Multiply,
                TokenType::Divide    => MultiplicativeOperator::Divide,
                TokenType::Modulo    => MultiplicativeOperator::Modulo,
                TokenType::IntDivide => MultiplicativeOperator::IntDivide,
                _ => panic!(),
            };
            self.advance();

            following.push((operator, self.unary_expression()?));
        }

        return Ok(MultiplicativeExpression { location: start_location, base, following });
    }

    fn unary_expression(&mut self) -> Result<UnaryExpression> {
        let start_location = self.current_token.location.clone();
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
            self.advance();
            return Ok(UnaryExpression::Operator {
                location: start_location,
                operator,
                expression: Box::new(self.unary_expression()?),
            });
        }

        return Ok(UnaryExpression::Power(Box::new(self.exponential_expression()?)));
    }

    fn exponential_expression(&mut self) -> Result<ExponentialExpression> {
        let start_location = self.current_token.location.clone();
        let base = self.atom()?;

        let mut exponent = None;
        if self.current_token.token_type == TokenType::Power {
            self.advance();

            exponent = Some(self.unary_expression()?)
        }

        return Ok(ExponentialExpression { location: start_location, base, exponent });
    }

    fn atom(&mut self) -> Result<Atom> {
        let start_location = self.current_token.location.clone();
        if self.current_token.matches(TokenType::Keyword, "null") {
            self.advance();

            return Ok(Atom::Null);
        }

        if self.current_token.token_type == TokenType::Number {
            let value = self.current_token.value.clone();
            let number = match value.parse::<Decimal>() {
                Ok(value) => value,
                Err(e) => match e {
                    Error::ErrorString(message) => error!(ValueError, start_location, "{}", message),
                    Error::ExceedsMaximumPossibleValue => error!(ValueError, start_location, "Value too high"),
                    Error::LessThanMinimumPossibleValue => error!(ValueError, start_location, "Value too low"),
                    Error::ScaleExceedsMaximumPrecision(_) => error!(ValueError, start_location, "Value too precise"),
                },
            };
            self.advance();
            return Ok(Atom::Number(number));
        }

        if self.current_token.matches(TokenType::Keyword, "true")
            || self.current_token.matches(TokenType::Keyword, "false") {
            let value = self.current_token.value == "true";
            self.advance();

            return Ok(Atom::Bool(value));
        }

        if self.current_token.token_type == TokenType::String {
            let value = self.current_token.value.clone();
            self.advance();

            return Ok(Atom::String(value));
        }

        if self.current_token.token_type == TokenType::Identifier {
            let value = self.current_token.value.clone();

            if self.next_token().token_type == TokenType::LParen {
                return Ok(Atom::Call(self.call_expression()?));
            } else {
                self.advance();
            }

            return Ok(Atom::Identifier { location: start_location, name: value });
        }

        if self.current_token.token_type == TokenType::LParen {
            self.advance();

            let expression = self.expression()?;

            expected!(self, RParen, "')'");
            self.advance();

            return Ok(Atom::Expression(expression));
        }

        syntax!(self, "Expected expression, found '{}'", self.current_token.value);
    }

    fn call_expression(&mut self) -> Result<CallExpression> {
        let start_location = self.current_token.location.clone();
        let identifier = self.current_token.value.clone();
        self.advance();

        let args = self.arguments()?;

        return Ok(CallExpression { location: start_location, identifier, args });
    }

    fn arguments(&mut self) -> Result<Vec<Expression>> {
        self.advance();

        let mut args = vec![];
        if self.current_token.token_type != TokenType::RParen {
            args.push(self.expression()?);

            while self.current_token.token_type == TokenType::Comma {
                self.advance();

                args.push(self.expression()?);
            }
        }

        expected!(self, RParen, "')'");
        self.advance();

        return Ok(args);
    }
}
