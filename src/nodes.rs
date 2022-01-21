use fsize::fsize;

#[derive(Debug)]
pub struct Statements { pub statements: Vec<Statement> }
#[derive(Debug)]
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
#[derive(Debug)]
pub struct DeclareStatement { pub identifier: String, pub expression: Expression }
#[derive(Debug)]
pub struct AssignStatement { pub identifier: String, pub operator: AssignOperator, pub expression: Expression }
#[derive(Debug)]
pub enum AssignOperator {
    Normal,   // =
    Plus,     // +=
    Minus,    // -=
    Multiply, // *=
    Divide,   // /=
}
#[derive(Debug)]
pub struct IfStatement { pub condition: Expression, pub block: Statements, pub else_block: Statements }
#[derive(Debug)]
pub struct LoopStatement { pub block: Statements }
#[derive(Debug)]
pub struct WhileStatement { pub condition: Expression, pub block: Statements }
#[derive(Debug)]
pub struct ForStatement { pub identifier: String, pub expression: Expression, pub block: Statements }
#[derive(Debug)]
pub struct FunctionDeclaration { pub identifier: String, pub params: Vec<String>, pub block: Statements }
#[derive(Debug)]
pub struct ReturnStatement { pub expression: Option<Expression> }

#[derive(Debug)]
pub struct Expression { pub base: Box<TernaryExpression>, pub range: Box<Option<(bool, TernaryExpression)>> }
#[derive(Debug)]
pub struct TernaryExpression { pub base: OrExpression, pub ternary: Option<(Expression, Expression)> }
#[derive(Debug)]
pub struct OrExpression { pub base: AndExpression, pub following: Vec<AndExpression> }
#[derive(Debug)]
pub struct AndExpression { pub base: EqualityExpression, pub following: Vec<EqualityExpression> }
#[derive(Debug)]
pub enum EqualityOperator {
    Equal,    // ==
    NotEqual, // !=
}
#[derive(Debug)]
pub struct EqualityExpression { pub base: RelationalExpression, pub following: Vec<(EqualityOperator, RelationalExpression)> }
#[derive(Debug)]
pub enum RelationalOperator {
    LessThan,           // <
    GreaterThan,        // >
    LessThanOrEqual,    // <=
    GreaterThanOrEqual, // >=
}
#[derive(Debug)]
pub struct RelationalExpression { pub base: AdditiveExpression, pub following: Vec<(RelationalOperator, AdditiveExpression)> }
#[derive(Debug)]
pub enum AdditiveOperator {
    Plus,  // +
    Minus, // -
}
#[derive(Debug)]
pub struct AdditiveExpression { pub base: MultiplicativeExpression, pub following: Vec<(AdditiveOperator, MultiplicativeExpression)> }
#[derive(Debug)]
pub enum MultiplicativeOperator {
    Multiply, // *
    Divide,   // /
}
#[derive(Debug)]
pub struct MultiplicativeExpression { pub base: UnaryExpression, pub following: Vec<(MultiplicativeOperator, UnaryExpression)> }
#[derive(Debug)]
pub enum UnaryOperator {
    Plus,  // +
    Minus, // -
    Not,   // !
}
#[derive(Debug)]
pub enum UnaryExpression {
    Operator(UnaryOperator, Box<UnaryExpression>),
    Expression(Expression),
    Atom(Atom),
}
#[derive(Debug)]
pub enum Atom {
    Number(Number),
    Bool(bool),
    String(String),
    Identifier(String),
    Call(CallExpression),
    Null,
}
#[derive(Debug)]
pub enum Number {
    Int(isize),
    Float(fsize),
}
#[derive(Debug)]
pub struct CallExpression { pub identifier: String, pub args: Vec<Expression>, pub following: Vec<(String, Vec<Expression>)> }
