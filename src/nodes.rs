use std::{fmt::Debug, ops::Deref, slice};

use crate::{error::Location, tokens::TokenType};
use rust_decimal::Decimal;

macro_rules! node {
    ($name:ident; $($field:ident : $type:ty),* $(,)?) => {
        #[derive(Debug)]
        pub struct $name {
            pub start: Location,
            pub end: Location,
            $(
                pub $field: $type,
            )*
        }
    };
}

/////////////////////////////////////////////

type Identifier = String;
type Opt<T> = Option<T>;

pub struct Rep<T>(Option<Vec<T>>);

impl<T> Rep<T> {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn push(&mut self, value: T) {
        if let Some(vec) = &mut self.0 {
            vec.push(value);
        } else {
            self.0 = Some(vec![value]);
        }
    }

    pub fn take(&mut self) -> Self {
        self.0.take().into()
    }

    pub fn unwrap_or_default(self) -> Vec<T> {
        self.0.unwrap_or_default()
    }
}

impl<T> Default for Rep<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<Option<Vec<T>>> for Rep<T> {
    fn from(val: Option<Vec<T>>) -> Self {
        Self(val)
    }
}

impl<T> Deref for Rep<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        match &self.0 {
            Some(vec) => vec,
            None => &[],
        }
    }
}

impl<T: Debug> Debug for Rep<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(vec) => vec.fmt(f),
            None => write!(f, "None"),
        }
    }
}

impl<'a, T> IntoIterator for &'a Rep<T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/////////////////////////////////////////////

pub type Program = Rep<Statement>;

#[derive(Debug)]
pub enum Block {
    Single(Box<Statement>),
    Multiple(BlockExpr),
}

#[derive(Debug)]
pub enum Statement {
    Var(VarStmt),
    Function(FunctionDecl),
    Class(ClassDecl),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Return(ReturnStmt),
    Expr(Expression),
}
node! { VarStmt; ident: Identifier, expr: Opt<Expression> }
node! { FunctionDecl; ident: Identifier, args: ArgNames, block: Block }
node! { ClassDecl; ident: Identifier, block: MemberBlock }
node! { BreakStmt; expr: Expression }
node! { ContinueStmt; }
node! { ReturnStmt; expr: Expression }

node! { Member; is_static: bool, member_type: MemberType }
#[derive(Debug)]
pub enum MemberType {
    Attribute(VarStmt),
    Method(FunctionDecl),
}
node! { MemberBlock; members: Rep<Member> }

pub type Expression = RangeExpr;
node! { RangeExpr; base: Box<OrExpr>, range: Opt<(TokenType, Box<OrExpr>)> }
node! { OrExpr; base: AndExpr, following: Rep<AndExpr> }
node! { AndExpr; base: BitOrExpr, following: Rep<BitOrExpr> }
node! { BitOrExpr; base: BitXorExpr, following: Rep<BitXorExpr> }
node! { BitXorExpr; base: BitAndExpr, following: Rep<BitAndExpr> }
node! { BitAndExpr; base: EqExpr, following: Rep<EqExpr> }
node! { EqExpr; left: RelExpr, right: Opt<(TokenType, RelExpr)> }
node! { RelExpr; left: ShiftExpr, right: Opt<(TokenType, ShiftExpr)> }
node! { ShiftExpr; base: AddExpr, following: Rep<(TokenType, AddExpr)> }
node! { AddExpr; base: MulExpr, following: Rep<(TokenType, MulExpr)> }
node! { MulExpr; base: UnaryExpr, following: Rep<(TokenType, UnaryExpr)> }
#[derive(Debug)]
pub enum UnaryExpr {
    Unary {
        start: Location,
        end: Location,
        operator: TokenType,
        expr: Box<UnaryExpr>,
    },
    Done(Box<ExpExpr>),
}
node! { ExpExpr; base: AssignExpr, exponent: Opt<UnaryExpr> }
node! { AssignExpr; left: CallExpr, right: Opt<(TokenType, Expression)> }
node! { CallExpr; base: MemberExpr, following: Opt<(Args, Rep<CallPart>)> }
node! { MemberExpr; base: Atom, following: Rep<MemberPart> }
#[derive(Debug)]
pub enum Atom {
    Number(Decimal),
    Bool(bool),
    String(String),
    Null,
    Identifier {
        start: Location,
        end: Location,
        name: Identifier,
    },
    Expr(Expression),
    IfExpr(IfExpr),
    ForExpr(ForExpr),
    WhileExpr(WhileExpr),
    LoopExpr(LoopExpr),
    FunExpr(FunExpr),
    ClassExpr(ClassExpr),
    TryExpr(TryExpr),
    BlockExpr(BlockExpr),
}
node! { IfExpr; cond: Expression, block: Block, else_block: Opt<Block> }
node! { ForExpr; ident: Identifier, iter: Expression, block: Block }
node! { WhileExpr; cond: Expression, block: Block }
node! { LoopExpr; block: Block }
node! { FunExpr; args: ArgNames, block: Block }
node! { ClassExpr; block: MemberBlock }
node! { TryExpr; try_block: Block, ident: Identifier, catch_block: Block }
node! { BlockExpr; stmts: Rep<Statement>, ending_semi: bool }

#[derive(Debug)]
pub enum MemberPart {
    Field(Identifier),
}
#[derive(Debug)]
pub enum CallPart {
    Member(MemberPart),
    Args(Args),
}
pub type Args = Rep<Expression>;
pub type ArgNames = Rep<Identifier>;
