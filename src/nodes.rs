use bigdecimal::BigDecimal;
use num_bigint::BigInt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Statements { pub statements: Vec<Statement> }
#[derive(Debug, PartialEq, Eq, Clone)]
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DeclareStatement { pub identifier: String, pub expression: Expression }
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AssignStatement { pub identifier: String, pub operator: AssignOperator, pub expression: Expression }
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AssignOperator {
    Normal,   // =
    Plus,     // +=
    Minus,    // -=
    Multiply, // *=
    Divide,   // /=
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IfStatement { pub condition: Expression, pub block: Statements, pub else_block: Statements }
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LoopStatement { pub block: Statements }
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WhileStatement { pub condition: Expression, pub block: Statements }
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ForStatement { pub identifier: String, pub expression: Expression, pub block: Statements }
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionDeclaration { pub identifier: String, pub params: Vec<String>, pub block: Statements }
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ReturnStatement { pub expression: Option<Expression> }

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Expression { pub base: Box<TernaryExpression>, pub range: Box<Option<(bool, TernaryExpression)>> }
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TernaryExpression { pub base: OrExpression, pub ternary: Option<(Expression, Expression)> }
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OrExpression { pub base: AndExpression, pub following: Vec<AndExpression> }
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AndExpression { pub base: EqualityExpression, pub following: Vec<EqualityExpression> }
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EqualityOperator {
    Equal,    // ==
    NotEqual, // !=
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EqualityExpression { pub base: RelationalExpression, pub other: Option<(EqualityOperator, RelationalExpression)> }
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RelationalOperator {
    LessThan,           // <
    GreaterThan,        // >
    LessThanOrEqual,    // <=
    GreaterThanOrEqual, // >=
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RelationalExpression { pub base: AdditiveExpression, pub other: Option<(RelationalOperator, AdditiveExpression)> }
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AdditiveOperator {
    Plus,  // +
    Minus, // -
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AdditiveExpression { pub base: MultiplicativeExpression, pub following: Vec<(AdditiveOperator, MultiplicativeExpression)> }
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MultiplicativeOperator {
    Multiply, // *
    Divide,   // /
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MultiplicativeExpression { pub base: ExponentialExpression, pub following: Vec<(MultiplicativeOperator, ExponentialExpression)> }
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExponentialExpression { pub base: UnaryExpression, pub following: Vec<UnaryExpression> }
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UnaryOperator {
    Plus,  // +
    Minus, // -
    Not,   // !
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UnaryExpression {
    Operator(UnaryOperator, Box<UnaryExpression>),
    Expression(Expression),
    Atom(Atom),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Atom {
    Number(Number),
    Bool(bool),
    String(String),
    Identifier(String),
    Call(CallExpression),
    Null,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Number {
    Int(BigInt),
    Float(BigDecimal),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CallExpression { pub identifier: String, pub args: Vec<Expression> }
