use rust_decimal::Decimal;
use crate::{error::Location, tokens::TokenType};

#[derive(Debug, PartialEq, Clone)]
pub struct Statements {
    pub start: Location,
    pub end: Location,
    pub statements: Vec<Statement>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Declare   (DeclareStatement),
    Assign    (AssignStatement),
    Loop      (LoopStatement),
    While     (WhileStatement),
    For       (ForStatement),
    Function  (FunctionDeclaration),
    Expression(Expression),
    Break,
    Continue,
    Return    (ReturnStatement),
}
#[derive(Debug, PartialEq, Clone)]
pub struct DeclareStatement {
    pub start: Location,
    pub end: Location,
    pub identifier: String,
    pub expression: Expression,
}
#[derive(Debug, PartialEq, Clone)]
pub struct AssignStatement {
    pub start: Location,
    pub end: Location,
    pub identifier: String,
    pub operator: TokenType,
    pub expression: Expression,
}
#[derive(Debug, PartialEq, Clone)]
pub struct LoopStatement {
    pub start: Location,
    pub end: Location,
    pub block: Statements,
}
#[derive(Debug, PartialEq, Clone)]
pub struct WhileStatement {
    pub start: Location,
    pub end: Location,
    pub condition: Expression,
    pub block: Statements,
}
#[derive(Debug, PartialEq, Clone)]
pub struct ForStatement {
    pub start: Location,
    pub end: Location,
    pub identifier: String,
    pub expression: Expression,
    pub block: Statements,
}
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDeclaration {
    pub start: Location,
    pub end: Location,
    pub identifier: String,
    pub params: Vec<String>,
    pub block: Statements,
}
#[derive(Debug, PartialEq, Clone)]
pub struct ReturnStatement {
    pub start: Location,
    pub end: Location,
    pub expression: Option<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expression {
    pub start: Location,
    pub end: Location,
    pub base: Box<OrExpression>,
    pub range: Box<Option<(bool, OrExpression)>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct OrExpression {
    pub start: Location,
    pub end: Location,
    pub base: AndExpression,
    pub following: Vec<AndExpression>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct AndExpression {
    pub start: Location,
    pub end: Location,
    pub base: EqualityExpression,
    pub following: Vec<EqualityExpression>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct EqualityExpression {
    pub start: Location,
    pub end: Location,
    pub base: RelationalExpression,
    pub other: Option<(TokenType, RelationalExpression)>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct RelationalExpression {
    pub start: Location,
    pub end: Location,
    pub base: AdditiveExpression,
    pub other: Option<(TokenType, AdditiveExpression)>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct AdditiveExpression {
    pub start: Location,
    pub end: Location,
    pub base: MultiplicativeExpression,
    pub following: Vec<(TokenType, MultiplicativeExpression)>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct MultiplicativeExpression {
    pub start: Location,
    pub end: Location,
    pub base: UnaryExpression,
    pub following: Vec<(TokenType, UnaryExpression)>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum UnaryExpression {
    Operator {
        start: Location,
        end: Location,
        operator: TokenType,
        expression: Box<UnaryExpression>,
    },
    Power(Box<ExponentialExpression>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct ExponentialExpression {
    pub start: Location,
    pub end: Location,
    pub base: CallExpression,
    pub exponent: Option<UnaryExpression>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum MemberPart {
    Identifier(String),
}
#[derive(Debug, PartialEq, Clone)]
pub enum CallPart {
    Member(MemberPart),
    Arguments(Vec<Expression>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct CallExpression {
    pub start: Location,
    pub end: Location,
    pub base: MemberExpression,
    pub call: Option<(Vec<Expression>, Vec<CallPart>)>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct MemberExpression {
    pub start: Location,
    pub end: Location,
    pub base: Atom,
    pub parts: Vec<MemberPart>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Atom {
    Number(Decimal),
    Bool(bool),
    String(String),
    Identifier { start: Location, end: Location, name: String },
    If(IfExpression),
    Fun(FunExpression),
    Null,
    Expression(Expression),
    Block(Statements),
}
#[derive(Debug, PartialEq, Clone)]
pub struct IfExpression {
    pub start: Location,
    pub end: Location,
    pub condition: Expression,
    pub block: Statements,
    pub else_block: Option<Statements>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct FunExpression {
    pub start: Location,
    pub end: Location,
    pub params: Vec<String>,
    pub block: Statements,
}
