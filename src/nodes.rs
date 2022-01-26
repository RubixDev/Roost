use rust_decimal::Decimal;

#[derive(Debug, PartialEq, Clone)]
pub struct Statements { pub statements: Vec<Statement> }
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
pub struct DeclareStatement { pub identifier: String, pub expression: Expression }
#[derive(Debug, PartialEq, Clone)]
pub struct AssignStatement { pub identifier: String, pub operator: AssignOperator, pub expression: Expression }
#[derive(Debug, PartialEq, Clone)]
pub enum AssignOperator {
    Normal,   // =
    Plus,     // +=
    Minus,    // -=
    Multiply, // *=
    Divide,   // /=
}
#[derive(Debug, PartialEq, Clone)]
pub struct IfStatement { pub condition: Expression, pub block: Statements, pub else_block: Statements }
#[derive(Debug, PartialEq, Clone)]
pub struct LoopStatement { pub block: Statements }
#[derive(Debug, PartialEq, Clone)]
pub struct WhileStatement { pub condition: Expression, pub block: Statements }
#[derive(Debug, PartialEq, Clone)]
pub struct ForStatement { pub identifier: String, pub expression: Expression, pub block: Statements }
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDeclaration { pub identifier: String, pub params: Vec<String>, pub block: Statements }
#[derive(Debug, PartialEq, Clone)]
pub struct ReturnStatement { pub expression: Option<Expression> }

#[derive(Debug, PartialEq, Clone)]
pub struct Expression { pub base: Box<TernaryExpression>, pub range: Box<Option<(bool, TernaryExpression)>> }
#[derive(Debug, PartialEq, Clone)]
pub struct TernaryExpression { pub base: OrExpression, pub ternary: Option<(Expression, Expression)> }
#[derive(Debug, PartialEq, Clone)]
pub struct OrExpression { pub base: AndExpression, pub following: Vec<AndExpression> }
#[derive(Debug, PartialEq, Clone)]
pub struct AndExpression { pub base: EqualityExpression, pub following: Vec<EqualityExpression> }
#[derive(Debug, PartialEq, Clone)]
pub enum EqualityOperator {
    Equal,    // ==
    NotEqual, // !=
}
#[derive(Debug, PartialEq, Clone)]
pub struct EqualityExpression { pub base: RelationalExpression, pub other: Option<(EqualityOperator, RelationalExpression)> }
#[derive(Debug, PartialEq, Clone)]
pub enum RelationalOperator {
    LessThan,           // <
    GreaterThan,        // >
    LessThanOrEqual,    // <=
    GreaterThanOrEqual, // >=
}
#[derive(Debug, PartialEq, Clone)]
pub struct RelationalExpression { pub base: AdditiveExpression, pub other: Option<(RelationalOperator, AdditiveExpression)> }
#[derive(Debug, PartialEq, Clone)]
pub enum AdditiveOperator {
    Plus,  // +
    Minus, // -
}
#[derive(Debug, PartialEq, Clone)]
pub struct AdditiveExpression { pub base: MultiplicativeExpression, pub following: Vec<(AdditiveOperator, MultiplicativeExpression)> }
#[derive(Debug, PartialEq, Clone)]
pub enum MultiplicativeOperator {
    Multiply, // *
    Divide,   // /
}
#[derive(Debug, PartialEq, Clone)]
pub struct MultiplicativeExpression { pub base: ExponentialExpression, pub following: Vec<(MultiplicativeOperator, ExponentialExpression)> }
#[derive(Debug, PartialEq, Clone)]
pub struct ExponentialExpression { pub base: UnaryExpression, pub following: Vec<UnaryExpression> }
#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOperator {
    Plus,  // +
    Minus, // -
    Not,   // !
}
#[derive(Debug, PartialEq, Clone)]
pub enum UnaryExpression {
    Operator(UnaryOperator, Box<UnaryExpression>),
    Expression(Expression),
    Atom(Atom),
}
#[derive(Debug, PartialEq, Clone)]
pub enum Atom {
    Number(Decimal),
    Bool(bool),
    String(String),
    Identifier(String),
    Call(CallExpression),
    Null,
}
#[derive(Debug, PartialEq, Clone)]
pub struct CallExpression { pub identifier: String, pub args: Vec<Expression> }
