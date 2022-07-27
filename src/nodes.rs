use std::fmt::Debug;

use crate::{error::Span, tokens::TokenKind};
use rust_decimal::Decimal;

macro_rules! node {
    ($name:ident; $($field:ident : $type:ty),* $(,)?) => {
        #[derive(Debug, PartialEq, Clone)]
        pub struct $name {
            pub span: Span,
            $(pub $field: $type,)*
        }
    };
}

pub type Program = Statements;
pub type Statements = Vec<Statement>;
pub type Block = Statements;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Var(VarStmt),
    Function(FunctionDecl),
    Class(ClassDecl),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Return(ReturnStmt),
    Expr(Expression),
}
node! { VarStmt; ident: String, expr: Option<Expression> }
node! { FunctionDecl; ident: String, args: Params, block: Block }
node! { ClassDecl; ident: String, block: MemberBlock }
node! { BreakStmt; expr: Option<Expression> }
node! { ContinueStmt; }
node! { ReturnStmt; expr: Option<Expression> }

node! { Member; is_static: bool, kind: MemberKind }
#[derive(Debug, PartialEq, Clone)]
pub enum MemberKind {
    Attribute(VarStmt),
    Method(FunctionDecl),
}
node! { MemberBlock; members: Vec<Member> }

pub type Expression = RangeExpr;
#[derive(Debug, PartialEq, Clone)]
pub enum RangeExpr {
    None(Box<OrExpr>),
    Closed(Box<OrExpr>, TokenKind, Box<OrExpr>, Span),
    OpenEnd(Box<OrExpr>, Span),
    OpenStart(TokenKind, Box<OrExpr>, Span),
    Open,
}
node! { OrExpr; base: AndExpr, following: Vec<AndExpr> }
node! { AndExpr; base: BitOrExpr, following: Vec<BitOrExpr> }
node! { BitOrExpr; base: BitXorExpr, following: Vec<BitXorExpr> }
node! { BitXorExpr; base: BitAndExpr, following: Vec<BitAndExpr> }
node! { BitAndExpr; base: EqExpr, following: Vec<EqExpr> }
node! { EqExpr; left: RelExpr, right: Option<(TokenKind, RelExpr)> }
node! { RelExpr; left: ShiftExpr, right: Option<(TokenKind, ShiftExpr)> }
node! { ShiftExpr; base: AddExpr, following: Vec<(TokenKind, AddExpr)> }
node! { AddExpr; base: MulExpr, following: Vec<(TokenKind, MulExpr)> }
node! { MulExpr; base: UnaryExpr, following: Vec<(TokenKind, UnaryExpr)> }
#[derive(Debug, PartialEq, Clone)]
pub enum UnaryExpr {
    Unary {
        span: Span,
        operator: TokenKind,
        expr: Box<UnaryExpr>,
    },
    Done(Box<ExpExpr>),
}
node! { ExpExpr; base: AssignExpr, exponent: Option<UnaryExpr> }
node! { AssignExpr; left: CallExpr, right: Option<(TokenKind, Expression)> }
node! { CallExpr; base: MemberExpr, following: Vec<CallPart> }
node! { MemberExpr; base: Atom, following: Vec<MemberPart> }
#[derive(Debug, PartialEq, Clone)]
pub enum Atom {
    Number(Decimal),
    Bool(bool),
    String(String),
    Null,
    Identifier { span: Span, name: String },
    Expr(Expression),
    List(ListLiteral),
    IfExpr(IfExpr),
    ForExpr(ForExpr),
    WhileExpr(WhileExpr),
    LoopExpr(LoopExpr),
    FunExpr(FunExpr),
    ClassExpr(ClassExpr),
    TryExpr(TryExpr),
    BlockExpr(BlockExpr),
}
pub type ListLiteral = Vec<Expression>;
node! { IfExpr; cond: Expression, block: Block, else_block: Option<Block> }
node! { ForExpr; ident: String, iter: Expression, block: Block }
node! { WhileExpr; cond: Expression, block: Block }
node! { LoopExpr; block: Block }
node! { FunExpr; args: Params, block: Block }
node! { ClassExpr; block: MemberBlock }
node! { TryExpr; try_block: Block, ident: String, catch_block: Block }
pub type BlockExpr = Block;

#[derive(Debug, PartialEq, Clone)]
pub enum MemberPart {
    Field(String),
    Index(Expression),
}
#[derive(Debug, PartialEq, Clone)]
pub enum CallPart {
    Member(MemberPart),
    Args(Args),
}
pub type Args = Vec<Expression>;
pub type Params = Vec<String>;
