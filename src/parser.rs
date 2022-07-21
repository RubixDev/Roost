use std::{mem, result};

use crate::{
    error::{Error, Result},
    lexer::Lexer,
    nodes::*,
    tokens::{Token, TokenKind},
};

macro_rules! syntax_err {
    ($self:ident, $($arg:tt)*) => {
        error!(SyntaxError, $self.curr_tok.start, $self.curr_tok.end, $($arg)*)
    };
}

macro_rules! expect {
    ($self:ident, Eol, $name:expr) => { expect!(@common $self, Eol, $name) };
    ($self:ident, $kind:ident, $name:expr) => {
        expect!(@common $self, $kind, $name);
        $self.advance();
    };
    (@common $self:ident, $kind:ident, $name:expr) => {
        if !of_kinds!($self, $kind) {
            $self.errors.push(error_val!(
                SyntaxError,
                $self.curr_tok.start,
                $self.curr_tok.end,
                "Expected {}, found '{}'",
                $name,
                $self.curr_tok.value(),
            ));
        }
    };
}

macro_rules! expect_ident {
    ($self:ident) => {{
        if !of_kinds!($self, Identifier) {
            $self.errors.push(error_val!(
                SyntaxError,
                $self.curr_tok.start,
                $self.curr_tok.end,
                "Expected identifier, found '{}'",
                $self.curr_tok.value(),
            ));
        }
        let ident = $self.curr_tok.value.take().unwrap_or_default();
        $self.advance();
        ident
    }};
}

macro_rules! of_kinds {
    ($self:ident, Eol) => {{
        of_kinds!(@skip $self);
        $self.prev_tok.kind == TokenKind::Eol
    }};
    ($self:ident, $kind:ident) => {{
        of_kinds!(@skip $self);
        $self.curr_tok.kind == TokenKind::$kind
    }};
    ($self:ident, $($kind:ident),+ $(,)?) => {{
        of_kinds!(@skip $self);
        [$(TokenKind::$kind, )*].contains(&$self.curr_tok.kind)
    }};
    (@skip $self:ident) => {
        while $self.curr_tok.kind == TokenKind::Eol {
            $self.advance();
        }
    };
}

macro_rules! simple_expr {
    ($name:ident -> $type:ident : $($tok:ident),+ => $next:ident $kind:tt) => {
        fn $name(&mut self) -> Result<$type> {
            let start = self.curr_tok.start;
            simple_expr!(@kind self, start, $type, $($tok),+ | $next, $kind)
        }
    };
    (@kind $self:ident, $start:ident, $type:ident, $($tok:ident),+ | $next:ident, *) => {{
        let base = $self.$next()?;
        let mut following = vec![];
        while of_kinds!($self, $($tok),+) {
            following.push(simple_expr!(@inner $self, $next, $($tok),+));
        }
        done!($type, $start, $self; base, following)
    }};
    (@kind $self:ident, $start:ident, $type:ident, $($tok:ident),+ | $next:ident, ?) => {{
        let left = $self.$next()?;
        let right = if of_kinds!($self, $($tok),+) {
            Some(simple_expr!(@inner $self, $next, $($tok),+))
        } else {
            None
        };
        done!($type, $start, $self; left, right)
    }};
    (@inner $self:ident, $next:ident, $_:ident) => {{
        $self.advance();
        $self.$next()?
    }};
    (@inner $self:ident, $next:ident, $($_:ident),+) => {{
        let tok = $self.curr_tok.kind;
        $self.advance();
        (tok, $self.$next()?)
    }};
}

macro_rules! done {
    ($type:ident, $start:ident, $self:ident; $($tt:tt)*) => {
        Ok($type {
            start: $start,
            end: $self.prev_tok.end,
            $($tt)*
        })
    };
}

pub struct Parser<'i> {
    lexer: Lexer<'i>,
    prev_tok: Token,
    curr_tok: Token,
    errors: Vec<Error>,
}

impl<'i> Parser<'i> {
    pub fn new(lexer: Lexer<'i>) -> Self {
        Self {
            lexer,
            prev_tok: Token::dummy(),
            curr_tok: Token::dummy(),
            errors: vec![],
        }
    }

    pub fn new_parse(lexer: Lexer<'i>) -> result::Result<Program, Vec<Error>> {
        let mut parser = Self::new(lexer);
        parser.parse()
    }

    pub fn parse(&mut self) -> result::Result<Program, Vec<Error>> {
        self.advance();
        let statements = match Self::program(self) {
            Ok(statements) => statements,
            Err(error) => {
                self.errors.push(error);
                return Err(mem::take(&mut self.errors));
            }
        };

        if !of_kinds!(self, Eof) {
            self.errors.push(error_val!(
                SyntaxError,
                self.curr_tok.start,
                self.curr_tok.end,
                "Expected EOF",
            ));
        }
        if !self.errors.is_empty() {
            return Err(mem::take(&mut self.errors));
        }
        Ok(statements)
    }

    fn advance(&mut self) {
        mem::swap(&mut self.prev_tok, &mut self.curr_tok);
        self.curr_tok = match self.lexer.next_token() {
            Ok(token) => token,
            Err((error, token)) => {
                self.errors.push(error);
                token
            }
        };
    }

    // ---------------------------------------

    #[inline]
    fn program(&mut self) -> Result<Program> {
        self.statements()
    }

    fn statements(&mut self) -> Result<Statements> {
        let start = self.curr_tok.start;

        let mut stmts = vec![];
        if !of_kinds!(self, RBrace, Eof) {
            stmts.push(self.statement()?);
            while of_kinds!(self, Eol) {
                if of_kinds!(self, RBrace, Eof) {
                    break;
                }
                stmts.push(self.statement()?);
            }
        }

        done!(Statements, start, self; stmts)
    }

    fn block(&mut self) -> Result<Block> {
        if of_kinds!(self, LBrace) {
            Ok(self.block_expr()?)
        } else {
            let stmt = self.statement()?;
            let (start, end) = match &stmt {
                Statement::Var(node) => (node.start, node.end),
                Statement::Function(node) => (node.start, node.end),
                Statement::Class(node) => (node.start, node.end),
                Statement::Break(node) => (node.start, node.end),
                Statement::Continue(node) => (node.start, node.end),
                Statement::Return(node) => (node.start, node.end),
                Statement::Expr(node) => (node.start, node.end),
            };
            Ok(Block {
                start,
                end,
                stmts: vec![stmt],
            })
        }
    }

    fn statement(&mut self) -> Result<Statement> {
        Ok(match self.curr_tok.kind {
            TokenKind::Var => Statement::Var(self.var_stmt()?),
            TokenKind::Fun => Statement::Function(self.function_decl()?),
            TokenKind::Class => Statement::Class(self.class_decl()?),
            TokenKind::Break => Statement::Break(self.break_stmt()?),
            TokenKind::Continue => Statement::Continue(self.continue_stmt()?),
            TokenKind::Return => Statement::Return(self.return_stmt()?),
            _ => Statement::Expr(self.expression()?),
        })
    }

    fn var_stmt(&mut self) -> Result<VarStmt> {
        let start = self.curr_tok.start;

        expect!(self, Var, "'var'");
        let ident = expect_ident!(self);

        let expr = if of_kinds!(self, Assign) {
            self.advance();
            Some(self.expression()?)
        } else {
            None
        };

        done!(VarStmt, start, self; ident, expr)
    }

    fn function_decl(&mut self) -> Result<FunctionDecl> {
        let start = self.curr_tok.start;

        expect!(self, Fun, "'fun'");
        let ident = expect_ident!(self);
        let args = self.params()?;
        let block = self.block()?;

        done!(FunctionDecl, start, self; ident, args, block)
    }

    fn class_decl(&mut self) -> Result<ClassDecl> {
        let start = self.curr_tok.start;

        expect!(self, Class, "'class'");
        let ident = expect_ident!(self);
        let block = self.member_block()?;

        done!(ClassDecl, start, self; ident, block)
    }

    fn break_stmt(&mut self) -> Result<BreakStmt> {
        let start = self.curr_tok.start;

        expect!(self, Break, "'break'");
        let expr = if !of_kinds!(self, Eof, Eol, RBrace) {
            Some(self.expression()?)
        } else {
            None
        };

        done!(BreakStmt, start, self; expr)
    }

    fn continue_stmt(&mut self) -> Result<ContinueStmt> {
        let start = self.curr_tok.start;

        expect!(self, Continue, "'continue'");

        done!(ContinueStmt, start, self;)
    }
    fn return_stmt(&mut self) -> Result<ReturnStmt> {
        let start = self.curr_tok.start;

        expect!(self, Return, "'return'");
        let expr = if !of_kinds!(self, Eof, Eol, RBrace) {
            Some(self.expression()?)
        } else {
            None
        };

        done!(ReturnStmt, start, self; expr)
    }

    fn member(&mut self) -> Result<Member> {
        let start = self.curr_tok.start;

        let is_static = of_kinds!(self, Static);
        if is_static {
            self.advance();
        }
        let kind = if of_kinds!(self, Var) {
            MemberKind::Attribute(self.var_stmt()?)
        } else {
            MemberKind::Method(self.function_decl()?)
        };

        done!(Member, start, self; is_static, kind)
    }

    fn member_block(&mut self) -> Result<MemberBlock> {
        let start = self.curr_tok.start;

        expect!(self, LBrace, "'{'");
        let mut members = vec![];
        while !of_kinds!(self, RBrace, Eof) {
            members.push(self.member()?);
            expect!(self, Eol, "';' or line break");
        }
        expect!(self, RBrace, "'}'");

        done!(MemberBlock, start, self; members)
    }

    #[inline]
    fn expression(&mut self) -> Result<Expression> {
        self.range_expr()
    }

    fn range_expr(&mut self) -> Result<RangeExpr> {
        let start = self.curr_tok.start;

        let left = Box::new(self.or_expr()?);
        let right = if of_kinds!(self, Dots, DotsInclusive) {
            let tok = self.curr_tok.kind;
            self.advance();
            Some((tok, Box::new(self.or_expr()?)))
        } else {
            None
        };

        done!(RangeExpr, start, self; left, right)
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
    simple_expr!(mul_expr -> MulExpr: Star, Slash, Rem, Backslash => unary_expr *);

    fn unary_expr(&mut self) -> Result<UnaryExpr> {
        let start = self.curr_tok.start;

        if of_kinds!(self, Plus, Minus, Not) {
            let operator = self.curr_tok.kind;
            self.advance();
            let expr = Box::new(self.unary_expr()?);
            Ok(UnaryExpr::Unary {
                start,
                end: self.prev_tok.end,
                operator,
                expr,
            })
        } else {
            Ok(UnaryExpr::Done(Box::new(self.exp_expr()?)))
        }
    }

    fn exp_expr(&mut self) -> Result<ExpExpr> {
        let start = self.curr_tok.start;

        let base = self.assign_expr()?;
        let exponent = if of_kinds!(self, Pow) {
            self.advance();
            Some(self.unary_expr()?)
        } else {
            None
        };

        done!(ExpExpr, start, self; base, exponent)
    }

    fn assign_expr(&mut self) -> Result<AssignExpr> {
        let start = self.curr_tok.start;

        let left = self.call_expr()?;
        let right = if of_kinds!(
            self,
            Assign,
            StarAssign,
            SlashAssign,
            BackslashAssign,
            RemAssign,
            PlusAssign,
            MinusAssign,
            ShiftLeftAssign,
            ShiftRightAssign,
            BitAndAssign,
            BitXorAssign,
            BitOrAssign,
        ) {
            let tok = self.curr_tok.kind;
            self.advance();
            Some((tok, self.expression()?))
        } else {
            None
        };

        done!(AssignExpr, start, self; left, right)
    }

    fn call_expr(&mut self) -> Result<CallExpr> {
        let start = self.curr_tok.start;

        let base = self.member_expr()?;
        let mut following = vec![];
        if of_kinds!(self, LParen) {
            following.push(CallPart::Args(self.args()?));
            while of_kinds!(self, LParen, Dot) {
                following.push(self.call_part()?);
            }
        }

        done!(CallExpr, start, self; base, following)
    }

    fn member_expr(&mut self) -> Result<MemberExpr> {
        let start = self.curr_tok.start;

        let base = self.atom()?;
        let mut following = vec![];
        while of_kinds!(self, Dot) {
            following.push(self.member_part()?);
        }

        done!(MemberExpr, start, self; base, following)
    }

    fn atom(&mut self) -> Result<Atom> {
        let start = self.curr_tok.start;

        Ok(match self.curr_tok.kind {
            TokenKind::Number => {
                let num = self.curr_tok.take_value();
                self.advance();
                Atom::Number(match num.parse() {
                    Ok(num) => num,
                    Err(rust_decimal::Error::ErrorString(msg)) => {
                        error!(ValueError, start, self.prev_tok.end, "{}", msg);
                    }
                    Err(rust_decimal::Error::ExceedsMaximumPossibleValue) => {
                        error!(ValueError, start, self.prev_tok.end, "Value too high");
                    }
                    Err(rust_decimal::Error::LessThanMinimumPossibleValue) => {
                        error!(ValueError, start, self.prev_tok.end, "Value too low");
                    }
                    Err(rust_decimal::Error::ScaleExceedsMaximumPrecision(_)) => {
                        error!(ValueError, start, self.prev_tok.end, "Value too precise");
                    }
                    Err(_) => error!(
                        ValueError,
                        start, self.prev_tok.end, "Failed to parse number"
                    ),
                })
            }
            TokenKind::True => {
                self.advance();
                Atom::Bool(true)
            }
            TokenKind::False => {
                self.advance();
                Atom::Bool(true)
            }
            TokenKind::String => {
                let str = self.curr_tok.take_value();
                self.advance();
                Atom::String(str)
            }
            TokenKind::Null => {
                self.advance();
                Atom::Null
            }
            TokenKind::Identifier => {
                let name = self.curr_tok.take_value();
                self.advance();
                Atom::Identifier {
                    start,
                    end: self.prev_tok.end,
                    name,
                }
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.expression()?;
                expect!(self, RParen, "')'");
                Atom::Expr(expr)
            }
            TokenKind::If => Atom::IfExpr(self.if_expr()?),
            TokenKind::For => Atom::ForExpr(self.for_expr()?),
            TokenKind::While => Atom::WhileExpr(self.while_expr()?),
            TokenKind::Loop => Atom::LoopExpr(self.loop_expr()?),
            TokenKind::Fun => Atom::FunExpr(self.fun_expr()?),
            TokenKind::Class => Atom::ClassExpr(self.class_expr()?),
            TokenKind::Try => Atom::TryExpr(self.try_expr()?),
            TokenKind::LBrace => Atom::BlockExpr(self.block_expr()?),
            _ => syntax_err!(
                self,
                "Expected expression, found '{}'",
                self.curr_tok.value()
            ),
        })
    }

    fn if_expr(&mut self) -> Result<IfExpr> {
        let start = self.curr_tok.start;

        expect!(self, If, "'if'");
        expect!(self, LParen, "'('");
        let cond = self.expression()?;
        expect!(self, RParen, "')'");
        let block = self.block()?;
        let else_block = if of_kinds!(self, Else) {
            self.advance();
            Some(self.block()?)
        } else {
            None
        };

        done!(IfExpr, start, self; cond, block, else_block)
    }

    fn for_expr(&mut self) -> Result<ForExpr> {
        let start = self.curr_tok.start;

        expect!(self, For, "'for'");
        expect!(self, LParen, "'('");
        let ident = expect_ident!(self);
        expect!(self, In, "'in'");
        let iter = self.expression()?;
        expect!(self, RParen, "')'");
        let block = self.block()?;

        done!(ForExpr, start, self; ident, iter, block)
    }

    fn while_expr(&mut self) -> Result<WhileExpr> {
        let start = self.curr_tok.start;

        expect!(self, While, "'while'");
        expect!(self, LParen, "'('");
        let cond = self.expression()?;
        expect!(self, RParen, "')'");
        let block = self.block()?;

        done!(WhileExpr, start, self; cond, block)
    }

    fn loop_expr(&mut self) -> Result<LoopExpr> {
        let start = self.curr_tok.start;

        expect!(self, Loop, "'loop'");
        let block = self.block()?;

        done!(LoopExpr, start, self; block)
    }

    fn fun_expr(&mut self) -> Result<FunExpr> {
        let start = self.curr_tok.start;

        expect!(self, Fun, "'fun'");
        let args = self.params()?;
        let block = self.block()?;

        done!(FunExpr, start, self; args, block)
    }

    fn class_expr(&mut self) -> Result<ClassExpr> {
        let start = self.curr_tok.start;

        expect!(self, Class, "'class'");
        let block = self.member_block()?;

        done!(ClassExpr, start, self; block)
    }

    fn try_expr(&mut self) -> Result<TryExpr> {
        let start = self.curr_tok.start;

        expect!(self, Try, "'try'");
        let try_block = self.block()?;
        expect!(self, Catch, "'catch'");
        expect!(self, LParen, "'('");
        let ident = expect_ident!(self);
        expect!(self, RParen, "')'");
        let catch_block = self.block()?;

        done!(TryExpr, start, self; try_block, ident, catch_block)
    }

    fn block_expr(&mut self) -> Result<BlockExpr> {
        let start = self.curr_tok.start;

        expect!(self, LBrace, "'{'");
        let stmts = self.statements()?;
        expect!(self, RBrace, "'}'");

        Ok(BlockExpr {
            start,
            end: self.prev_tok.end,
            ..stmts
        })
    }

    fn member_part(&mut self) -> Result<MemberPart> {
        Ok(match self.curr_tok.kind {
            TokenKind::Dot => {
                self.advance();
                MemberPart::Field(expect_ident!(self))
            }
            _ => error!(
                SyntaxError,
                self.curr_tok.start,
                self.curr_tok.end,
                "Expected '.', found '{}'",
                self.curr_tok.value(),
            ),
        })
    }

    fn call_part(&mut self) -> Result<CallPart> {
        Ok(if of_kinds!(self, LParen) {
            CallPart::Args(self.args()?)
        } else {
            CallPart::Member(self.member_part()?)
        })
    }

    fn args(&mut self) -> Result<Args> {
        let mut args = vec![];
        expect!(self, LParen, "'('");
        if !of_kinds!(self, RParen) {
            args.push(self.expression()?);
            while of_kinds!(self, Comma) {
                self.advance();
                if of_kinds!(self, RParen) {
                    break;
                }
                args.push(self.expression()?);
            }
        }
        expect!(self, RParen, "')'");
        Ok(args)
    }

    fn params(&mut self) -> Result<Params> {
        let mut args = vec![];
        expect!(self, LParen, "'('");
        if !of_kinds!(self, RParen) {
            args.push(expect_ident!(self));
            while of_kinds!(self, Comma) {
                self.advance();
                if of_kinds!(self, RParen) {
                    break;
                }
                args.push(expect_ident!(self));
            }
        }
        expect!(self, RParen, "')'");
        Ok(args)
    }
}
