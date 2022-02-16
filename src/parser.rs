use rust_decimal::Decimal;
use crate::{
    tokens::{Token, TokenType},
    error::{Result, Error},
    nodes::{
        Statements,
        Statement,
        DeclareStatement,
        AssignStatement,
        IfExpression,
        LoopStatement,
        WhileStatement,
        ForStatement,
        FunctionDeclaration,
        ReturnStatement,
        Expression,
        OrExpression,
        AndExpression,
        EqualityExpression,
        RelationalExpression,
        AdditiveExpression,
        MultiplicativeExpression,
        UnaryExpression,
        Atom,
        CallExpression,
        ExponentialExpression,
        FunExpression,
        CallPart,
        MemberPart,
        MemberExpression,
    }, lexer::Lexer,
};

macro_rules! loc {
    ($self:ident) => {
        $self.current_token.start.clone()
    };
}

macro_rules! syntax {
    ($self:ident, $($arg:tt)*) => {
        error!(SyntaxError, $self.current_token.start.clone(), $self.current_token.end.clone(), $($arg)*)
    };
}

macro_rules! expected {
    ($self:ident, $token_type:ident, $name:expr) => {
        if $self.current_token.token_type != TokenType::$token_type {
            $self.errors.push(error_val!(
                SyntaxError,
                $self.current_token.start.clone(),
                $self.current_token.end.clone(),
                "Expected {}, found '{}'",
                $name,
                $self.current_token.value
            ));
        }
    };
}

macro_rules! is_type {
    ($self:ident, $type:ident) => {
        $self.current_token.token_type == TokenType::$type
    };
}

macro_rules! of_types {
    ($self:ident, $($type:ident),+) => {
        [$(TokenType::$type, )*].contains(&$self.current_token.token_type)
    };
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    errors: Vec<Error>,
}

impl <'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        return Parser {
            lexer,
            current_token: Token::dummy(),
            errors: vec![],
        };
    }

    pub fn new_parse(lexer: Lexer<'a>) -> std::result::Result<Statements, Vec<Error>> {
        let mut parser = Self::new(lexer);
        return parser.parse();
    }

    pub fn parse(&mut self) -> std::result::Result<Statements, Vec<Error>> {
        self.advance();
        let statements = match self.statements() {
            Ok(statements) => statements,
            Err(error) => {
                self.errors.push(error);
                return Err(self.errors.clone());
            },
        };

        if self.current_token.token_type != TokenType::EOF {
            self.errors.push(error_val!(SyntaxError, self.current_token.start.clone(), self.current_token.end.clone(), "Expected EOF"));
            return Err(self.errors.clone());
        }
        if !self.errors.is_empty() {
            return Err(self.errors.clone());
        }
        return Ok(statements);
    }

    fn advance(&mut self) {
        self.current_token = match self.lexer.next_token() {
            Ok(token) => token,
            Err((error, token)) => {
                self.errors.push(error);
                token
            },
        }
    }

    fn following_token(&self) -> Token {
        return self.lexer
            .clone()
            .next_token()
            .unwrap_or(Token::dummy());
    }

    // ---------------------------------------

    fn statements(&mut self) -> Result<Statements> {
        let start_location = loc!(self);
        while is_type!(self, EOL) {
            self.advance()
        }

        if ![TokenType::EOF, TokenType::RBrace].contains(&self.current_token.token_type) {
            let mut statements: Vec<Statement> = vec![];
            statements.push(self.statement()?);
            loop {
                if [TokenType::EOF, TokenType::RBrace].contains(&self.current_token.token_type) { break; }
                expected!(self, EOL, "';' or line break");
                while is_type!(self, EOL) {
                    self.advance();
                }
                if [TokenType::EOF, TokenType::RBrace].contains(&self.current_token.token_type) { break; }
                statements.push(self.statement()?);
            }
            return Ok(Statements { start: start_location, end: loc!(self), statements });
        }

        return Ok(Statements { start: start_location, end: loc!(self), statements: vec![] });
    }

    fn block(&mut self) -> Result<Statements> {
        let start_location = loc!(self);
        while is_type!(self, EOL) {
            self.advance();
        }

        let statements: Vec<Statement>;
        if is_type!(self, LBrace) {
            self.advance();

            statements = self.statements()?.statements;

            expected!(self, RBrace, "'}'");
            self.advance();
        } else {
            statements = vec![self.statement()?];
        }

        return Ok(Statements { start: start_location, end: loc!(self), statements });
    }

    fn statement(&mut self) -> Result<Statement> {
        if is_type!(self, Var) {
            return Ok(Statement::Declare(self.declare_statement()?));
        } else if is_type!(self, Loop) {
            return Ok(Statement::Loop(self.loop_statement()?));
        } else if is_type!(self, While) {
            return Ok(Statement::While(self.while_statement()?));
        } else if is_type!(self, For) {
            return Ok(Statement::For(self.for_statement()?));
        } else if is_type!(self, Fun) && self.following_token().token_type != TokenType::LParen {
            return Ok(Statement::Function(self.function_declaration()?));
        } else if is_type!(self, Break) {
            self.advance();
            return Ok(Statement::Break);
        } else if is_type!(self, Continue) {
            self.advance();
            return Ok(Statement::Continue);
        } else if is_type!(self, Return) {
            return Ok(Statement::Return(self.return_statement()?));
        } else if is_type!(self, Identifier)
            && [
                TokenType::Assign,
                TokenType::PlusAssign,
                TokenType::MinusAssign,
                TokenType::MultiplyAssign,
                TokenType::DivideAssign,
                TokenType::ModuloAssign,
                TokenType::IntDivideAssign,
                TokenType::PowerAssign,
            ].contains(&self.following_token().token_type) {
            return Ok(Statement::Assign(self.assign_statement()?));
        } else {
            return Ok(Statement::Expression(self.expression()?));
        }
    }

    fn declare_statement(&mut self) -> Result<DeclareStatement> {
        let start_location = loc!(self);
        self.advance();

        expected!(self, Identifier, "identifier");
        let identifier = self.current_token.value.clone();
        self.advance();

        expected!(self, Assign, "'='");
        self.advance();

        let expression = self.expression()?;

        return Ok(DeclareStatement { start: start_location, end: loc!(self), identifier, expression }) ;
    }

    fn assign_statement(&mut self) -> Result<AssignStatement> {
        let start_location = loc!(self);
        let identifier = self.current_token.value.clone();
        self.advance();

        let operator = self.current_token.token_type.clone();
        self.advance();

        let expression = self.expression()?;

        return Ok(AssignStatement { start: start_location, end: loc!(self), identifier, operator, expression });
    }

    fn if_expression(&mut self) -> Result<IfExpression> {
        let start_location = loc!(self);
        self.advance();

        expected!(self, LParen, "'('");
        self.advance();

        let condition = self.expression()?;

        expected!(self, RParen, "')'");
        self.advance();

        let block = self.block()?;
        let else_block: Option<Statements>;

        let mut lexer = self.lexer.clone();
        let mut current_token = self.current_token.clone();
        let mut count = 0;

        while current_token.token_type == TokenType::EOL {
            current_token = lexer.next_token().unwrap_or(Token::dummy());
            count += 1;
        }

        if current_token.token_type == TokenType::Else {
            for _ in 0..count { self.advance(); }
            self.advance();

            else_block = Some(self.block()?);
        } else {
            else_block = None;
        }

        return Ok(IfExpression { start: start_location, end: loc!(self), condition, block, else_block });
    }

    fn loop_statement(&mut self) -> Result<LoopStatement> {
        let start_location = loc!(self);
        self.advance();

        let block = self.block()?;

        return Ok(LoopStatement { start: start_location, end: loc!(self), block });
    }

    fn while_statement(&mut self) -> Result<WhileStatement> {
        let start_location = loc!(self);
        self.advance();

        expected!(self, LParen, "'('");
        self.advance();

        let condition = self.expression()?;

        expected!(self, RParen, "')'");
        self.advance();

        let block = self.block()?;

        return Ok(WhileStatement { start: start_location, end: loc!(self), condition, block });
    }

    fn for_statement(&mut self) -> Result<ForStatement> {
        let start_location = loc!(self);
        self.advance();

        expected!(self, LParen, "'('");
        self.advance();

        expected!(self, Identifier, "identifier");
        let identifier = self.current_token.value.clone();
        self.advance();

        expected!(self, In, "'in'");
        self.advance();

        let expression = self.expression()?;

        expected!(self, RParen, "')'");
        self.advance();

        let block = self.block()?;

        return Ok(ForStatement { start: start_location, end: loc!(self), identifier, expression, block });
    }

    fn function_declaration(&mut self) -> Result<FunctionDeclaration> {
        let start_location = loc!(self);
        self.advance();

        expected!(self, Identifier, "identifier");
        let identifier = self.current_token.value.clone();
        self.advance();

        let params = self.argument_names()?;

        let block = self.block()?;

        return Ok(FunctionDeclaration { start: start_location, end: loc!(self), identifier, params, block });
    }

    fn return_statement(&mut self) -> Result<ReturnStatement> {
        let start_location = loc!(self);
        self.advance();

        let mut expression = None;
        if ![TokenType::EOL, TokenType::EOF, TokenType::RBrace].contains(&self.current_token.token_type) {
            expression = Some(self.expression()?);
        }

        return Ok(ReturnStatement { start: start_location, end: loc!(self), expression });
    }


    fn expression(&mut self) -> Result<Expression> {
        let start_location = loc!(self);
        let base = self.or_expression()?;

        let mut range = None;
        if is_type!(self, RangeDots) {
            let inclusive = self.current_token.value == "..=";
            self.advance();

            let upper = self.or_expression()?;

            range = Some((inclusive, upper));
        }

        return Ok(Expression { start: start_location, end: loc!(self), base: Box::new(base), range: Box::new(range) });
    }

    fn or_expression(&mut self) -> Result<OrExpression> {
        let start_location = loc!(self);
        let base = self.and_expression()?;

        let mut following = vec![];
        while is_type!(self, Or) {
            self.advance();

            following.push(self.and_expression()?);
        }

        return Ok(OrExpression { start: start_location, end: loc!(self), base, following });
    }

    fn and_expression(&mut self) -> Result<AndExpression> {
        let start_location = loc!(self);
        let base = self.equality_expression()?;

        let mut following = vec![];
        while is_type!(self, And) {
            self.advance();

            following.push(self.equality_expression()?);
        }

        return Ok(AndExpression { start: start_location, end: loc!(self), base, following });
    }

    fn equality_expression(&mut self) -> Result<EqualityExpression> {
        let start_location = loc!(self);
        let base = self.relational_expression()?;

        let mut other = None;
        if [
            TokenType::Equal,
            TokenType::NotEqual,
        ].contains(&self.current_token.token_type) {
            let operator = self.current_token.token_type.clone();
            self.advance();

            other = Some((operator, self.relational_expression()?));
        }

        return Ok(EqualityExpression { start: start_location, end: loc!(self), base, other });
    }

    fn relational_expression(&mut self) -> Result<RelationalExpression> {
        let start_location = loc!(self);
        let base = self.additive_expression()?;

        let mut other = None;
        if [
            TokenType::LessThan,
            TokenType::GreaterThan,
            TokenType::LessThanOrEqual,
            TokenType::GreaterThanOrEqual,
        ].contains(&self.current_token.token_type) {
            let operator = self.current_token.token_type.clone();
            self.advance();

            other = Some((operator, self.additive_expression()?));
        }

        return Ok(RelationalExpression { start: start_location, end: loc!(self), base, other });
    }

    fn additive_expression(&mut self) -> Result<AdditiveExpression> {
        let start_location = loc!(self);
        let base = self.multiplicative_expression()?;

        let mut following = vec![];
        while [
            TokenType::Plus,
            TokenType::Minus,
        ].contains(&self.current_token.token_type) {
            let operator = self.current_token.token_type.clone();
            self.advance();

            following.push((operator, self.multiplicative_expression()?));
        }

        return Ok(AdditiveExpression { start: start_location, end: loc!(self), base, following });
    }

    fn multiplicative_expression(&mut self) -> Result<MultiplicativeExpression> {
        let start_location = loc!(self);
        let base = self.unary_expression()?;

        let mut following = vec![];
        while [
            TokenType::Multiply,
            TokenType::Divide,
            TokenType::Modulo,
            TokenType::IntDivide,
        ].contains(&self.current_token.token_type) {
            let operator = self.current_token.token_type.clone();
            self.advance();

            following.push((operator, self.unary_expression()?));
        }

        return Ok(MultiplicativeExpression { start: start_location, end: loc!(self), base, following });
    }

    fn unary_expression(&mut self) -> Result<UnaryExpression> {
        let start_location = loc!(self);
        if [
            TokenType::Plus,
            TokenType::Minus,
            TokenType::Not,
        ].contains(&self.current_token.token_type) {
            let operator = self.current_token.token_type.clone();
            self.advance();
            return Ok(UnaryExpression::Operator {
                start: start_location,
                end: loc!(self),
                operator,
                expression: Box::new(self.unary_expression()?),
            });
        }

        return Ok(UnaryExpression::Power(Box::new(self.exponential_expression()?)));
    }

    fn exponential_expression(&mut self) -> Result<ExponentialExpression> {
        let start_location = loc!(self);
        let base = self.call_expression()?;

        let mut exponent = None;
        if is_type!(self, Power) {
            self.advance();

            exponent = Some(self.unary_expression()?)
        }

        return Ok(ExponentialExpression { start: start_location, end: loc!(self), base, exponent });
    }

    fn call_expression(&mut self) -> Result<CallExpression> {
        let start_location = loc!(self);
        let base = self.member_expression()?;

        if is_type!(self, LParen) {
            let args = self.arguments()?;

            let mut parts = vec![];
            while of_types!(self, Dot, LParen) {
                if is_type!(self, Dot) {
                    self.advance();

                    expected!(self, Identifier, "identifier");
                    let name = self.current_token.value.clone();
                    self.advance();

                    parts.push(CallPart::Member(MemberPart::Identifier(name)));
                } else {
                    parts.push(CallPart::Arguments(self.arguments()?));
                }
            }

            return Ok(CallExpression { start: start_location, end: loc!(self), base, call: Some((args, parts)) });
        }

        return Ok(CallExpression { start: start_location, end: loc!(self), base, call: None });
    }

    fn member_expression(&mut self) -> Result<MemberExpression> {
        let start_location = loc!(self);
        let base = self.atom()?;

        let mut parts = vec![];
        while is_type!(self, Dot) {
            self.advance();

            expected!(self, Identifier, "identifier");
            let name = self.current_token.value.clone();
            self.advance();

            parts.push(MemberPart::Identifier(name));
        }

        return Ok(MemberExpression { start: start_location, end: loc!(self), base, parts });
    }

    fn atom(&mut self) -> Result<Atom> {
        let start_location = loc!(self);
        if is_type!(self, Null) {
            self.advance();

            return Ok(Atom::Null);
        }

        if is_type!(self, If) {
            return Ok(Atom::If(self.if_expression()?));
        }

        if is_type!(self, Fun) {
            return Ok(Atom::Fun(self.fun_expression()?));
        }

        if is_type!(self, Number) {
            let value = self.current_token.value.clone();
            let number = match value.parse::<Decimal>() {
                Ok(value) => value,
                Err(e) => match e {
                    rust_decimal::Error::ErrorString(message)            => error!(ValueError, start_location, loc!(self), "{}", message),
                    rust_decimal::Error::ExceedsMaximumPossibleValue     => error!(ValueError, start_location, loc!(self), "Value too high"),
                    rust_decimal::Error::LessThanMinimumPossibleValue    => error!(ValueError, start_location, loc!(self), "Value too low"),
                    rust_decimal::Error::ScaleExceedsMaximumPrecision(_) => error!(ValueError, start_location, loc!(self), "Value too precise"),
                },
            };
            self.advance();
            return Ok(Atom::Number(number));
        }

        if is_type!(self, True)
            || is_type!(self, False) {
            let value = self.current_token.value == "true";
            self.advance();

            return Ok(Atom::Bool(value));
        }

        if is_type!(self, String) {
            let value = self.current_token.value.clone();
            self.advance();

            return Ok(Atom::String(value));
        }

        if is_type!(self, Identifier) {
            let value = self.current_token.value.clone();
            self.advance();

            return Ok(Atom::Identifier { start: start_location, end: loc!(self), name: value });
        }

        if is_type!(self, LParen) {
            self.advance();

            let expression = self.expression()?;

            expected!(self, RParen, "')'");
            self.advance();

            return Ok(Atom::Expression(expression));
        }

        if is_type!(self, LBrace) {
            self.advance();

            let block = self.statements()?;

            expected!(self, RBrace, "'}'");
            self.advance();

            return Ok(Atom::Block(block));
        }

        syntax!(self, "Expected expression, found '{}'", self.current_token.value);
    }

    fn fun_expression(&mut self) -> Result<FunExpression> {
        let start_location = loc!(self);
        self.advance();

        let params = self.argument_names()?;

        let block = self.block()?;

        return Ok(FunExpression { start: start_location, end: loc!(self), params, block });
    }

    fn arguments(&mut self) -> Result<Vec<Expression>> {
        expected!(self, LParen, "'('");
        self.advance();

        let mut args = vec![];
        if self.current_token.token_type != TokenType::RParen {
            args.push(self.expression()?);

            while is_type!(self, Comma) {
                self.advance();

                args.push(self.expression()?);
            }
        }

        expected!(self, RParen, "')'");
        self.advance();

        return Ok(args);
    }

    fn argument_names(&mut self) -> Result<Vec<String>> {
        expected!(self, LParen, "'('");
        self.advance();

        let mut args = vec![];
        if self.current_token.token_type != TokenType::RParen {
            expected!(self, Identifier, "identifier");
            args.push(self.current_token.value.clone());
            self.advance();

            while is_type!(self, Comma) {
                self.advance();

                expected!(self, Identifier, "identifier");
                args.push(self.current_token.value.clone());
                self.advance();
            }
        }

        expected!(self, RParen, "')'");
        self.advance();

        return Ok(args);
    }
}
