use std::result;

use crate::{
    error::{Error, Result},
    lexer::Lexer,
    nodes::*,
    tokens::{Token, TokenType},
};

macro_rules! syntax_err {
    ($self:ident, $($arg:tt)*) => {
        error!(SyntaxError, $self.token.start, $self.token.end, $($arg)*)
    };
}

macro_rules! expect {
    ($self:ident, $token_type:ident, $name:expr) => {
        if $self.token.token_type != TokenType::$token_type {
            $self.errors.push(error_val!(
                SyntaxError,
                $self.token.start,
                $self.token.end,
                "Expected {}, found '{}'",
                $name,
                $self.token.value(),
            ));
        }
        $self.advance();
    };
}

macro_rules! expect_ident {
    ($self:ident) => {{
        if $self.token.token_type != TokenType::Identifier {
            $self.errors.push(error_val!(
                SyntaxError,
                $self.token.start,
                $self.token.end,
                "Expected identifier, found '{}'",
                $self.token.value(),
            ));
        }
        let ident = $self.token.value.take().unwrap_or_default();
        $self.advance();
        ident
    }};
}

macro_rules! of_types {
    ($self:ident, $type:ident) => {
        $self.token.token_type == TokenType::$type
    };
    ($self:ident, $($type:ident),+ $(,)?) => {
        [$(TokenType::$type, )*].contains(&$self.token.token_type)
    };
}

macro_rules! simple_expr {
    ($name:ident -> $type:ident : $($tok:ident),+ => $next:ident $kind:tt) => {
        fn $name(&mut self) -> Result<'f, $type<'f>> {
            let start = self.token.start;
            simple_expr!(@kind self, start, $type, $($tok),+ | $next, $kind)
        }
    };
    (@kind $self:ident, $start:ident, $type:ident, $($tok:ident),+ | $next:ident, *) => {{
        let base = $self.$next()?;
        let mut following = Rep::new();
        while of_types!($self, $($tok),+) {
            following.push(simple_expr!(@inner $self, $next, $($tok),+));
        }
        Ok($type { start: $start, end: $self.token.start, base, following })
    }};
    (@kind $self:ident, $start:ident, $type:ident, $($tok:ident),+ | $next:ident, ?) => {{
        let left = $self.$next()?;
        let right = if of_types!($self, $($tok),+) {
            Some(simple_expr!(@inner $self, $next, $($tok),+))
        } else {
            None
        };
        Ok($type { start: $start, end: $self.token.start, left, right })
    }};
    (@inner $self:ident, $next:ident, $_:ident) => {{
        $self.advance();
        $self.$next()?
    }};
    (@inner $self:ident, $next:ident, $($_:ident),+) => {{
        let tok = $self.token.token_type;
        $self.advance();
        (tok, $self.$next()?)
    }};
}

macro_rules! done {
    ($type:ident, $start:ident, $self:ident; $($tt:tt)*) => {
        Ok($type {
            start: $start,
            end: $self.token.start,
            $($tt)*
        })
    };
}

pub struct Parser<'i, 'f> {
    lexer: Lexer<'i, 'f>,
    token: Token<'f>,
    errors: Rep<Error<'f>>,
}

impl<'i, 'f> Parser<'i, 'f> {
    pub fn new(lexer: Lexer<'i, 'f>) -> Self {
        return Parser {
            lexer,
            token: Token::dummy(),
            errors: Rep::new(),
        };
    }

    pub fn new_parse(lexer: Lexer<'i, 'f>) -> result::Result<Program<'f>, Vec<Error<'f>>> {
        let mut parser = Self::new(lexer);
        parser.parse()
    }

    pub fn parse(&mut self) -> result::Result<Program<'f>, Vec<Error<'f>>> {
        self.advance();
        let statements = match Self::program(self) {
            Ok(statements) => statements,
            Err(error) => {
                self.errors.push(error);
                return Err(self.errors.take().unwrap_or_default());
            }
        };

        if !of_types!(self, Eof) {
            self.errors.push(error_val!(
                SyntaxError,
                self.token.start,
                self.token.end,
                "Expected EOF",
            ));
        }
        if !self.errors.is_empty() {
            return Err(self.errors.take().unwrap_or_default());
        }
        Ok(statements)
    }

    fn advance(&mut self) {
        self.token = match self.lexer.next_token() {
            Ok(token) => token,
            Err((error, token)) => {
                self.errors.push(error);
                token
            }
        }
    }

    // ---------------------------------------

    fn program(&mut self) -> Result<'f, Program<'f>> {
        let mut stmts = Rep::new();
        while !of_types!(self, Eof) {
            match self.statement() {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => self.errors.push(e),
            }
            expect!(self, Semicolon, "';'");
        }
        Ok(stmts)
    }

    fn block(&mut self) -> Result<'f, Block<'f>> {
        if of_types!(self, LBrace) {
            Ok(Block::Multiple(self.block_expr()?))
        } else {
            Ok(Block::Single(Box::new(self.statement()?)))
        }
    }

    fn statement(&mut self) -> Result<'f, Statement<'f>> {
        Ok(match self.token.token_type {
            TokenType::Var => Statement::Var(self.var_stmt()?),
            TokenType::Fun => Statement::Function(self.function_decl()?),
            TokenType::Class => Statement::Class(self.class_decl()?),
            TokenType::Break => Statement::Break(self.break_stmt()?),
            TokenType::Continue => Statement::Continue(self.continue_stmt()?),
            TokenType::Return => Statement::Return(self.return_stmt()?),
            _ => Statement::Expr(self.expression()?),
        })
    }

    fn var_stmt(&mut self) -> Result<'f, VarStmt<'f>> {
        let start = self.token.start;

        expect!(self, Var, "'var'");
        let ident = expect_ident!(self);

        let expr = if of_types!(self, Assign) {
            self.advance();
            Some(self.expression()?)
        } else {
            None
        };

        done!(VarStmt, start, self; ident, expr)
    }

    fn function_decl(&mut self) -> Result<'f, FunctionDecl<'f>> {
        let start = self.token.start;

        expect!(self, Fun, "'fun'");
        let ident = expect_ident!(self);
        let args = self.arg_names()?;
        let block = self.block()?;

        done!(FunctionDecl, start, self; ident, args, block)
    }

    fn class_decl(&mut self) -> Result<'f, ClassDecl<'f>> {
        let start = self.token.start;

        expect!(self, Class, "'class'");
        let ident = expect_ident!(self);
        let block = self.member_block()?;

        done!(ClassDecl, start, self; ident, block)
    }

    fn break_stmt(&mut self) -> Result<'f, BreakStmt<'f>> {
        let start = self.token.start;

        expect!(self, Break, "'break'");
        let expr = self.expression()?;

        done!(BreakStmt, start, self; expr)
    }

    fn continue_stmt(&mut self) -> Result<'f, ContinueStmt<'f>> {
        let start = self.token.start;

        expect!(self, Continue, "'continue'");

        done!(ContinueStmt, start, self;)
    }
    fn return_stmt(&mut self) -> Result<'f, ReturnStmt<'f>> {
        let start = self.token.start;

        expect!(self, Return, "'return'");
        let expr = self.expression()?;

        done!(ReturnStmt, start, self; expr)
    }

    fn member(&mut self) -> Result<'f, Member<'f>> {
        let start = self.token.start;

        let is_static = of_types!(self, Static);
        if is_static {
            self.advance();
        }
        let member_type = if of_types!(self, Var) {
            MemberType::Attribute(self.var_stmt()?)
        } else {
            MemberType::Method(self.function_decl()?)
        };

        done!(Member, start, self; is_static, member_type)
    }

    fn member_block(&mut self) -> Result<'f, MemberBlock<'f>> {
        let start = self.token.start;

        expect!(self, LBrace, "'{'");
        let mut members = Rep::new();
        while !of_types!(self, RBrace, Eof) {
            match self.member() {
                Ok(member) => members.push(member),
                Err(e) => self.errors.push(e),
            }
        }
        expect!(self, RBrace, "'}'");

        done!(MemberBlock, start, self; members)
    }

    fn expression(&mut self) -> Result<'f, Expression<'f>> {
        self.range_expr()
    }

    fn range_expr(&mut self) -> Result<'f, RangeExpr<'f>> {
        let start = self.token.start;

        let base = Box::new(self.or_expr()?);
        let range = if of_types!(self, Dots, DotsInclusive) {
            let tok = self.token.token_type;
            self.advance();
            Some((tok, Box::new(self.or_expr()?)))
        } else {
            None
        };

        done!(RangeExpr, start, self; base, range)
    }

    simple_expr!(or_expr -> OrExpr: Or => and_expr *);
    simple_expr!(and_expr -> AndExpr: And => bit_or_expr *);
    simple_expr!(bit_or_expr -> BitOrExpr: BitOr => bit_xor_expr *);
    simple_expr!(bit_xor_expr -> BitXorExpr: BitXor => bit_and_expr *);
    simple_expr!(bit_and_expr -> BitAndExpr: BitAnd => eq_expr *);
    simple_expr!(eq_expr -> EqExpr: Equal, NotEqual => rel_expr ?);
    simple_expr!(rel_expr -> RelExpr: LessThan, LessThanOrEqual, GreaterThan, GreaterThanOrEqual => shift_expr ?);
    simple_expr!(shift_expr -> ShiftExpr: ShiftLeft, ShiftRight => add_expr *);
    simple_expr!(add_expr -> AddExpr: Plus, Minus => mul_expr *);
    simple_expr!(mul_expr -> MulExpr: Multiply, Divide, Modulo, IntDivide => unary_expr *);

    fn unary_expr(&mut self) -> Result<'f, UnaryExpr<'f>> {
        let start = self.token.start;

        if of_types!(self, Plus, Minus, Not) {
            let operator = self.token.token_type;
            self.advance();
            let expr = Box::new(self.unary_expr()?);
            Ok(UnaryExpr::Unary {
                start,
                end: self.token.start,
                operator,
                expr,
            })
        } else {
            Ok(UnaryExpr::Done(Box::new(self.exp_expr()?)))
        }
    }

    fn exp_expr(&mut self) -> Result<'f, ExpExpr<'f>> {
        let start = self.token.start;

        let base = self.assign_expr()?;
        let exponent = if of_types!(self, Power) {
            self.advance();
            Some(self.unary_expr()?)
        } else {
            None
        };

        done!(ExpExpr, start, self; base, exponent)
    }

    fn assign_expr(&mut self) -> Result<'f, AssignExpr<'f>> {
        let start = self.token.start;

        let left = self.call_expr()?;
        let right = if of_types!(
            self,
            Assign,
            PlusAssign,
            MultiplyAssign,
            DivideAssign,
            IntDivideAssign,
            ModuloAssign,
            PlusAssign,
            MinusAssign,
            ShiftLeftAssign,
            ShiftRightAssign,
            BitAndAssign,
            BitXorAssign,
            BitOrAssign,
        ) {
            let tok = self.token.token_type;
            self.advance();
            Some((tok, self.expression()?))
        } else {
            None
        };

        done!(AssignExpr, start, self; left, right)
    }

    fn call_expr(&mut self) -> Result<'f, CallExpr<'f>> {
        let start = self.token.start;

        let base = self.member_expr()?;
        let following = if of_types!(self, LParen) {
            let args = self.args()?;
            let mut parts = Rep::new();
            while of_types!(self, LParen, Dot) {
                parts.push(self.call_part()?);
            }
            Some((args, parts))
        } else {
            None
        };

        done!(CallExpr, start, self; base, following)
    }

    fn member_expr(&mut self) -> Result<'f, MemberExpr<'f>> {
        let start = self.token.start;

        let base = self.atom()?;
        let mut following = Rep::new();
        while of_types!(self, Dot) {
            following.push(self.member_part()?);
        }

        done!(MemberExpr, start, self; base, following)
    }

    fn atom(&mut self) -> Result<'f, Atom<'f>> {
        let start = self.token.start;

        Ok(match self.token.token_type {
            TokenType::Number => {
                let num = self.token.take_value();
                self.advance();
                Atom::Number(match num.parse() {
                    Ok(num) => num,
                    Err(rust_decimal::Error::ErrorString(msg)) => {
                        error!(ValueError, start, self.token.start, "{}", msg);
                    }
                    Err(rust_decimal::Error::ExceedsMaximumPossibleValue) => {
                        error!(ValueError, start, self.token.start, "Value too high");
                    }
                    Err(rust_decimal::Error::LessThanMinimumPossibleValue) => {
                        error!(ValueError, start, self.token.start, "Value too low");
                    }
                    Err(rust_decimal::Error::ScaleExceedsMaximumPrecision(_)) => {
                        error!(ValueError, start, self.token.start, "Value too precise");
                    }
                    Err(_) => error!(
                        ValueError,
                        start, self.token.start, "Failed to parse number"
                    ),
                })
            }
            TokenType::True => {
                self.advance();
                Atom::Bool(true)
            }
            TokenType::False => {
                self.advance();
                Atom::Bool(true)
            }
            TokenType::String => {
                let str = self.token.take_value();
                self.advance();
                Atom::String(str)
            }
            TokenType::Null => {
                self.advance();
                Atom::Null
            }
            TokenType::Identifier => {
                let name = self.token.take_value();
                self.advance();
                Atom::Identifier {
                    start,
                    end: self.token.start,
                    name,
                }
            }
            TokenType::LParen => {
                self.advance();
                let expr = self.expression()?;
                expect!(self, RParen, "')'");
                Atom::Expr(expr)
            }
            TokenType::If => Atom::IfExpr(self.if_expr()?),
            TokenType::For => Atom::ForExpr(self.for_expr()?),
            TokenType::While => Atom::WhileExpr(self.while_expr()?),
            TokenType::Loop => Atom::LoopExpr(self.loop_expr()?),
            TokenType::Fun => Atom::FunExpr(self.fun_expr()?),
            TokenType::Class => Atom::ClassExpr(self.class_expr()?),
            TokenType::Try => Atom::TryExpr(self.try_expr()?),
            TokenType::LBrace => Atom::BlockExpr(self.block_expr()?),
            _ => syntax_err!(self, "Expected expression, found '{}'", self.token.value()),
        })
    }

    fn if_expr(&mut self) -> Result<'f, IfExpr<'f>> {
        let start = self.token.start;

        expect!(self, If, "'if'");
        expect!(self, LParen, "'('");
        let cond = self.expression()?;
        expect!(self, RParen, "')'");
        let block = self.block()?;
        let else_block = if of_types!(self, Else) {
            self.advance();
            Some(self.block()?)
        } else {
            None
        };

        done!(IfExpr, start, self; cond, block, else_block)
    }

    fn for_expr(&mut self) -> Result<'f, ForExpr<'f>> {
        let start = self.token.start;

        expect!(self, For, "'for'");
        expect!(self, LParen, "'('");
        let ident = expect_ident!(self);
        expect!(self, In, "'in'");
        let iter = self.expression()?;
        expect!(self, RParen, "')'");
        let block = self.block()?;

        done!(ForExpr, start, self; ident, iter, block)
    }

    fn while_expr(&mut self) -> Result<'f, WhileExpr<'f>> {
        let start = self.token.start;

        expect!(self, While, "'while'");
        expect!(self, LParen, "'('");
        let cond = self.expression()?;
        expect!(self, RParen, "')'");
        let block = self.block()?;

        done!(WhileExpr, start, self; cond, block)
    }

    fn loop_expr(&mut self) -> Result<'f, LoopExpr<'f>> {
        let start = self.token.start;

        expect!(self, Loop, "'loop'");
        let block = self.block()?;

        done!(LoopExpr, start, self; block)
    }

    fn fun_expr(&mut self) -> Result<'f, FunExpr<'f>> {
        let start = self.token.start;

        expect!(self, Fun, "'fun'");
        let args = self.arg_names()?;
        let block = self.block()?;

        done!(FunExpr, start, self; args, block)
    }

    fn class_expr(&mut self) -> Result<'f, ClassExpr<'f>> {
        let start = self.token.start;

        expect!(self, Class, "'class'");
        let block = self.member_block()?;

        done!(ClassExpr, start, self; block)
    }

    fn try_expr(&mut self) -> Result<'f, TryExpr<'f>> {
        let start = self.token.start;

        expect!(self, Try, "'try'");
        let try_block = self.block()?;
        expect!(self, Catch, "'catch'");
        expect!(self, LParen, "'('");
        let ident = expect_ident!(self);
        expect!(self, RParen, "')'");
        let catch_block = self.block()?;

        done!(TryExpr, start, self; try_block, ident, catch_block)
    }

    fn block_expr(&mut self) -> Result<'f, BlockExpr<'f>> {
        let start = self.token.start;

        expect!(self, LBrace, "'{'");
        let mut stmts = Rep::new();
        let mut ending_semi = false;
        if !of_types!(self, RBrace, Eof) {
            stmts.push(self.statement()?);
            while of_types!(self, Semicolon) {
                self.advance();
                if of_types!(self, RBrace, Eof) {
                    ending_semi = true;
                    break;
                }
                stmts.push(self.statement()?);
            }
        }
        expect!(self, RBrace, "'}'");

        done!(BlockExpr, start, self; stmts, ending_semi)
    }

    fn member_part(&mut self) -> Result<'f, MemberPart> {
        Ok(match self.token.token_type {
            TokenType::Dot => {
                self.advance();
                MemberPart::Field(expect_ident!(self))
            }
            _ => error!(
                SyntaxError,
                self.token.start,
                self.token.end,
                "Expected '.', found '{}'",
                self.token.value(),
            ),
        })
    }

    fn call_part(&mut self) -> Result<'f, CallPart<'f>> {
        Ok(if of_types!(self, LParen) {
            CallPart::Args(self.args()?)
        } else {
            CallPart::Member(self.member_part()?)
        })
    }

    fn args(&mut self) -> Result<'f, Args<'f>> {
        let mut args = Rep::new();
        expect!(self, LParen, "'('");
        if !of_types!(self, RParen) {
            args.push(self.expression()?);
            while of_types!(self, Comma) {
                self.advance();
                if of_types!(self, RParen) {
                    break;
                }
                args.push(self.expression()?);
            }
        }
        expect!(self, RParen, "')'");
        Ok(args)
    }

    fn arg_names(&mut self) -> Result<'f, ArgNames> {
        let mut args = Rep::new();
        expect!(self, LParen, "'('");
        if !of_types!(self, RParen) {
            args.push(expect_ident!(self));
            while of_types!(self, Comma) {
                self.advance();
                if of_types!(self, RParen) {
                    break;
                }
                args.push(expect_ident!(self));
            }
        }
        expect!(self, RParen, "')'");
        Ok(args)
    }
}
