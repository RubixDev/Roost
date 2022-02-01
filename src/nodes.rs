use rust_decimal::Decimal;
use crate::error::Location;

#[derive(Debug, PartialEq, Clone)]
pub struct Statements {
    pub location: Location,
    pub statements: Vec<Statement>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Declare   (DeclareStatement),
    Assign    (AssignStatement),
    If        (IfStatement),
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
    pub location: Location,
    pub identifier: String,
    pub expression: Expression,
}
#[derive(Debug, PartialEq, Clone)]
pub struct AssignStatement {
    pub location: Location,
    pub identifier: String,
    pub operator: AssignOperator,
    pub expression: Expression,
}
#[derive(Debug, PartialEq, Clone)]
pub enum AssignOperator {
    Normal,    // =
    Plus,      // +=
    Minus,     // -=
    Multiply,  // *=
    Divide,    // /=
    Modulo,    // %=
    IntDivide, // \=
}
#[derive(Debug, PartialEq, Clone)]
pub struct IfStatement {
    pub location: Location,
    pub condition: Expression,
    pub block: Statements,
    pub else_block: Statements,
}
#[derive(Debug, PartialEq, Clone)]
pub struct LoopStatement {
    pub location: Location,
    pub block: Statements,
}
#[derive(Debug, PartialEq, Clone)]
pub struct WhileStatement {
    pub location: Location,
    pub condition: Expression,
    pub block: Statements,
}
#[derive(Debug, PartialEq, Clone)]
pub struct ForStatement {
    pub location: Location,
    pub identifier: String,
    pub expression: Expression,
    pub block: Statements,
}
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDeclaration {
    pub location: Location,
    pub identifier: String,
    pub params: Vec<String>,
    pub block: Statements,
}
#[derive(Debug, PartialEq, Clone)]
pub struct ReturnStatement {
    pub location: Location,
    pub expression: Option<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expression {
    pub location: Location,
    pub base: Box<TernaryExpression>,
    pub range: Box<Option<(bool, TernaryExpression)>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct TernaryExpression {
    pub location: Location,
    pub base: OrExpression,
    pub ternary: Option<(Expression, Expression)>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct OrExpression {
    pub location: Location,
    pub base: AndExpression,
    pub following: Vec<AndExpression>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct AndExpression {
    pub location: Location,
    pub base: EqualityExpression,
    pub following: Vec<EqualityExpression>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum EqualityOperator {
    Equal,    // ==
    NotEqual, // !=
}
#[derive(Debug, PartialEq, Clone)]
pub struct EqualityExpression {
    pub location: Location,
    pub base: RelationalExpression,
    pub other: Option<(EqualityOperator, RelationalExpression)>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum RelationalOperator {
    LessThan,           // <
    GreaterThan,        // >
    LessThanOrEqual,    // <=
    GreaterThanOrEqual, // >=
}
#[derive(Debug, PartialEq, Clone)]
pub struct RelationalExpression {
    pub location: Location,
    pub base: AdditiveExpression,
    pub other: Option<(RelationalOperator, AdditiveExpression)>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum AdditiveOperator {
    Plus,  // +
    Minus, // -
}
#[derive(Debug, PartialEq, Clone)]
pub struct AdditiveExpression {
    pub location: Location,
    pub base: MultiplicativeExpression,
    pub following: Vec<(AdditiveOperator, MultiplicativeExpression)>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum MultiplicativeOperator {
    Multiply,  // *
    Divide,    // /
    Modulo,    // %
    IntDivide, // \
}
#[derive(Debug, PartialEq, Clone)]
pub struct MultiplicativeExpression {
    pub location: Location,
    pub base: UnaryExpression,
    pub following: Vec<(MultiplicativeOperator, UnaryExpression)>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOperator {
    Plus,  // +
    Minus, // -
    Not,   // !
}
#[derive(Debug, PartialEq, Clone)]
pub enum UnaryExpression {
    Operator {
        location: Location,
        operator: UnaryOperator,
        expression: Box<UnaryExpression>,
    },
    Power(Box<ExponentialExpression>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct ExponentialExpression {
    pub location: Location,
    pub base: Atom,
    pub exponent: Option<UnaryExpression>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Atom {
    Number(Decimal),
    Bool(bool),
    String(String),
    Identifier { location: Location, name: String },
    Call(CallExpression),
    Null,
    Expression(Expression),
}
#[derive(Debug, PartialEq, Clone)]
pub struct CallExpression {
    pub location: Location,
    pub identifier: String,
    pub args: Vec<Expression>,
}
