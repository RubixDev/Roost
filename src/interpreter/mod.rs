mod value;
mod runtime_result;
mod built_in;

use std::collections::HashMap;
use runtime_result::RuntimeResult;
use rust_decimal::Decimal;
use value::{Value, types::type_of, types::Type};
use crate::error::{Result, Location};
use crate::nodes::{
    Statements,
    Statement,
    DeclareStatement,
    AssignStatement,
    AssignOperator,
    IfStatement,
    LoopStatement,
    WhileStatement,
    ForStatement,
    FunctionDeclaration,
    ReturnStatement,
    Expression,
    OrExpression,
    AndExpression,
    EqualityExpression,
    EqualityOperator,
    RelationalExpression,
    RelationalOperator,
    AdditiveExpression,
    AdditiveOperator,
    MultiplicativeExpression,
    MultiplicativeOperator,
    UnaryExpression,
    UnaryOperator,
    Atom,
    TernaryExpression,
    CallExpression,
    ExponentialExpression,
};
use self::value::{
    truth::Truth,
    calculative_operations::CalculativeOperations,
    relational_operations::RelationalOperations, iterator::ToIterator,
};

pub struct Interpreter {
    start_node: Statements,
    scopes: Vec<HashMap<String, Value>>,
    current_scope_index: usize,
}

impl Interpreter {
    pub fn new(start_node: Statements) -> Self {
        return Interpreter {
            start_node,
            scopes: vec![HashMap::from([
                (String::from("print"), Value::BuiltIn),
                (String::from("printl"), Value::BuiltIn),
                (String::from("typeOf"), Value::BuiltIn),
                (String::from("answer"), Value::Number(Decimal::from(42))),
            ])],
            current_scope_index: 0
        };
    }

    pub fn run(&mut self) -> Result<RuntimeResult> {
        return self.visit_statements(&self.start_node.clone(), true);
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.current_scope_index += 1;
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
        self.current_scope_index -= 1;
    }

    fn current_scope(&mut self) -> &mut HashMap<String, Value> {
        return &mut self.scopes[self.current_scope_index];
    }

    fn find_var(&self, name: &String, location: Location) -> Result<&Value> {
        let mut scope = self.current_scope_index;
        loop {
            if self.scopes[scope].contains_key(name) {
                return Ok(self.scopes[scope].get(name).unwrap());
            }
            if scope == 0 {
                error!(ReferenceError, location, "Variable or function with name '{}' not found", name);
            }
            scope -= 1;
        }
    }

    fn find_var_scope(&self, name: &String, location: Location) -> Result<usize> {
        let mut scope = self.current_scope_index;
        loop {
            if scope == 0 {
                error!(ReferenceError, location, "Variable or function with name '{}' not found", name);
            }
            if self.scopes[scope].contains_key(name) {
                return Ok(scope);
            }
            scope -= 1;
        }
    }

    // -----------------------------------------------

    fn visit_statements(&mut self, node: &Statements, new_scope: bool) -> Result<RuntimeResult> {
        let mut result = RuntimeResult::new();
        if new_scope { self.push_scope(); }
        for statement in &node.statements {
            result.register(self.visit_statement(&statement)?);
            if result.should_return() { break; }
        }
        self.pop_scope();
        return Ok(result);
    }

    fn visit_statement(&mut self, node: &Statement) -> Result<RuntimeResult> {
        return match node {
            Statement::Declare   (node) => self.visit_declare_statement(node),
            Statement::Assign    (node) => self.visit_assign_statement(node),
            Statement::If        (node) => self.visit_if_statement(node),
            Statement::Loop      (node) => self.visit_loop_statement(node),
            Statement::While     (node) => self.visit_while_statement(node),
            Statement::For       (node) => self.visit_for_statement(node),
            Statement::Function  (node) => Ok(self.visit_function_declaration(node)),
            Statement::Expression(node) => self.visit_expression(node),
            Statement::Break            => Ok(self.visit_break_statement()),
            Statement::Continue         => Ok(self.visit_continue_statement()),
            Statement::Return    (node) => self.visit_return_statement(node),
        };
    }

    fn visit_declare_statement(&mut self, node: &DeclareStatement) -> Result<RuntimeResult> {
        let mut result = self.visit_expression(&node.expression)?;
        if result.should_return() { return Ok(result); }

        self.current_scope().insert(node.identifier.clone(), result.value.clone().unwrap());
        result.success(None);
        return Ok(result);
    }

    fn visit_assign_statement(&mut self, node: &AssignStatement) -> Result<RuntimeResult> {
        let mut result = self.visit_expression(&node.expression)?;
        if result.should_return() { return Ok(result); }
        let new_value = result.value.clone().unwrap();
        let value_scope = self.find_var_scope(&node.identifier, node.location.clone())?;
        let value = self.scopes[value_scope][&node.identifier].clone();

        let old_type = type_of(&value);
        let new_type = type_of(&new_value);
        if old_type != Type::Null && new_type != Type::Null && old_type != new_type {
            error!(TypeError, node.location.clone(), "Expected type '{}', got '{}'", old_type, new_type);
        }

        self.scopes[value_scope].insert(node.identifier.clone(), match node.operator {
            AssignOperator::Normal   => Ok(new_value.clone()),
            AssignOperator::Plus     => value.plus(&new_value, node.location.clone()),
            AssignOperator::Minus    => value.minus(&new_value, node.location.clone()),
            AssignOperator::Multiply => value.multiply(&new_value, node.location.clone()),
            AssignOperator::Divide   => value.divide(&new_value, node.location.clone()),
        }?);

        result.success(None);
        return Ok(result);
    }

    fn visit_if_statement(&mut self, node: &IfStatement) -> Result<RuntimeResult> {
        let mut result = self.visit_expression(&node.condition)?;
        if result.should_return() { return Ok(result); }
        let condition = result.value.clone().unwrap();

        if condition.is_true() {
            result.register(self.visit_statements(&node.block, true)?)
        } else {
            result.register(self.visit_statements(&node.else_block, true)?)
        }

        if !result.should_return() { result.success(None); }
        return Ok(result);
    }

    fn visit_loop_statement(&mut self, node: &LoopStatement) -> Result<RuntimeResult> {
        let mut result = RuntimeResult::new();

        loop {
            result.register(self.visit_statements(&node.block, true)?);
            if result.should_continue { continue; }
            if result.should_break { break; }
            if result.should_return() { return Ok(result); }
        }

        result.success(None);
        return Ok(result);
    }

    fn visit_while_statement(&mut self, node: &WhileStatement) -> Result<RuntimeResult> {
        let mut result = RuntimeResult::new();

        loop {
            result.register(self.visit_expression(&node.condition)?);
            if result.should_return() { return Ok(result); }
            if !result.value.clone().unwrap().is_true() { break; }

            result.register(self.visit_statements(&node.block, true)?);
            if result.should_continue { continue; }
            if result.should_break { break; }
            if result.should_return() { return Ok(result); }
        }

        result.success(None);
        return Ok(result);
    }

    fn visit_for_statement(&mut self, node: &ForStatement) -> Result<RuntimeResult> {
        let mut result = self.visit_expression(&node.expression)?;
        if result.should_return() { return Ok(result); }
        let expression = result.value.clone().unwrap();

        let mut iter = expression.to_iter(node.location.clone())?;
        while let Some(i) = iter.next() {
            self.push_scope();
            self.current_scope().insert(node.identifier.clone(), i.clone());

            result.register(self.visit_statements(&node.block, false)?);
            if result.should_continue { continue; }
            if result.should_break { break; }
            if result.should_return() { return Ok(result); }
        }

        result.success(None);
        return Ok(result);
    }

    fn visit_function_declaration(&mut self, node: &FunctionDeclaration) -> RuntimeResult {
        self.current_scope().insert(node.identifier.clone(), Value::Function(node.params.clone(), node.block.clone()));
        return RuntimeResult::new();
    }

    fn visit_break_statement(&mut self) -> RuntimeResult {
        let mut result = RuntimeResult::new();
        result.success_break();
        return result;
    }

    fn visit_continue_statement(&mut self) -> RuntimeResult {
        let mut result = RuntimeResult::new();
        result.success_continue();
        return result;
    }

    fn visit_return_statement(&mut self, node: &ReturnStatement) -> Result<RuntimeResult> {
        let mut result = RuntimeResult::new();
        if let Some(expression) = &node.expression {
            result.register(self.visit_expression(&expression)?);
            if result.should_return() { return Ok(result); }
            result.success_return(result.value.clone());
        } else {
            result.success_return(Some(Value::Void));
        }
        return Ok(result);
    }


    fn visit_expression(&mut self, node: &Expression) -> Result<RuntimeResult> {
        let mut result = self.visit_ternary_expression(&*node.base)?;
        if result.should_return() { return Ok(result); }
        let val1 = result.value.clone().unwrap();

        if let Some((inclusive, expression)) = &*node.range {
            result.register(self.visit_ternary_expression(&expression)?);
            if result.should_return() { return Ok(result); }
            let val2 = result.value.clone().unwrap();

            let range = match val1 {
                Value::Number(start) => match val2 {
                    Value::Number(end) => {
                        if !start.fract().is_zero() || !end.fract().is_zero() {
                            error!(ValueError, node.location.clone(), "Range bounds have to be integers");
                        }
                        if !inclusive && start != end {
                            if start > end {
                                Value::Range(start, end + Decimal::ONE)
                            } else {
                                Value::Range(start, end - Decimal::ONE)
                            }
                        } else { Value::Range(start, end) }
                    },
                    _ => error!(TypeError, node.location.clone(), "Range bounds have to be of type number, got {}", type_of(&val1)),
                },
                _ => error!(TypeError, node.location.clone(), "Range bounds have to be of type number, got {}", type_of(&val1)),
            };
            result.success(Some(range));
        } else {
            result.success(Some(val1));
        }

        return Ok(result);
    }

    fn visit_ternary_expression(&mut self, node: &TernaryExpression) -> Result<RuntimeResult> {
        let mut result = self.visit_or_expression(&node.base)?;
        if result.should_return() { return Ok(result); }
        let condition = result.value.clone().unwrap();

        if let Some((if_expr, else_expr)) = &node.ternary {
            if condition.is_true() {
                result.register(self.visit_expression(&if_expr)?);
            } else {
                result.register(self.visit_expression(&else_expr)?);
            }
            if result.should_return() { return Ok(result); }
            result.success(result.value.clone());
        } else {
            result.success(Some(condition));
        }

        return Ok(result);
    }

    fn visit_or_expression(&mut self, node: &OrExpression) -> Result<RuntimeResult> {
        let mut result = self.visit_and_expression(&node.base)?;
        if result.should_return() { return Ok(result); }
        let base = result.value.clone().unwrap();

        if !node.following.is_empty() {
            if base.is_true() {
                result.success(Some(Value::Bool(true)));
                return Ok(result);
            }
            for expression in &node.following {
                result.register(self.visit_and_expression(&expression)?);
                if result.should_return() { return Ok(result); }
                if result.value.clone().unwrap().is_true() {
                    result.success(Some(Value::Bool(true)));
                    return Ok(result);
                }
            }
            result.success(Some(Value::Bool(false)));
        } else {
            result.success(Some(base));
        }

        return Ok(result);
    }

    fn visit_and_expression(&mut self, node: &AndExpression) -> Result<RuntimeResult> {
        let mut result = self.visit_equality_expression(&node.base)?;
        if result.should_return() { return Ok(result); }
        let base = result.value.clone().unwrap();

        if !node.following.is_empty() {
            if base.is_false() {
                result.success(Some(Value::Bool(false)));
                return Ok(result);
            }
            for expression in &node.following {
                result.register(self.visit_equality_expression(&expression)?);
                if result.should_return() { return Ok(result); }
                if result.value.clone().unwrap().is_false() {
                    result.success(Some(Value::Bool(false)));
                    return Ok(result);
                }
            }
            result.success(Some(Value::Bool(true)));
        } else {
            result.success(Some(base));
        }

        return Ok(result);
    }

    fn visit_equality_expression(&mut self, node: &EqualityExpression) -> Result<RuntimeResult> {
        let mut result = self.visit_relational_expression(&node.base)?;
        if result.should_return() { return Ok(result); }
        let base = result.value.clone().unwrap();

        if let Some((operator, expression)) = &node.other {
            result.register(self.visit_relational_expression(&expression)?);
            if result.should_return() { return Ok(result); }
            let equal = base == result.value.clone().unwrap();
            let out = match operator {
                EqualityOperator::Equal    =>  equal,
                EqualityOperator::NotEqual => !equal,
            };
            result.success(Some(Value::Bool(out)));
        } else {
            result.success(Some(base));
        }

        return Ok(result);
    }

    fn visit_relational_expression(&mut self, node: &RelationalExpression) -> Result<RuntimeResult> {
        let mut result = self.visit_additive_expression(&node.base)?;
        if result.should_return() { return Ok(result); }
        let base = result.value.clone().unwrap();

        if let Some((operator, expression)) = &node.other {
            result.register(self.visit_additive_expression(&expression)?);
            if result.should_return() { return Ok(result); }
            let other = result.value.clone().unwrap();

            let out = match operator {
                RelationalOperator::LessThan           => base.less_than(&other, node.location.clone()),
                RelationalOperator::GreaterThan        => base.greater_than(&other, node.location.clone()),
                RelationalOperator::LessThanOrEqual    => base.less_than_or_equal(&other, node.location.clone()),
                RelationalOperator::GreaterThanOrEqual => base.greater_than_or_equal(&other, node.location.clone()),
            }?;
            result.success(Some(out));
        } else {
            result.success(Some(base));
        }

        return Ok(result);
    }

    fn visit_additive_expression(&mut self, node: &AdditiveExpression) -> Result<RuntimeResult> {
        let mut result = self.visit_multiplicative_expression(&node.base)?;
        if result.should_return() { return Ok(result); }
        let mut base = result.value.clone().unwrap();

        for (operator, expression) in &node.following {
            result.register(self.visit_multiplicative_expression(&expression)?);
            if result.should_return() { return Ok(result); }
            let other = result.value.clone().unwrap();

            base = match operator {
                AdditiveOperator::Plus  => base.plus(&other, node.location.clone()),
                AdditiveOperator::Minus => base.minus(&other, node.location.clone()),
            }?;
        }

        result.success(Some(base));
        return Ok(result);
    }

    fn visit_multiplicative_expression(&mut self, node: &MultiplicativeExpression) -> Result<RuntimeResult> {
        let mut result = self.visit_unary_expression(&node.base)?;
        if result.should_return() { return Ok(result); }
        let mut base = result.value.clone().unwrap();

        for (operator, expression) in &node.following {
            result.register(self.visit_unary_expression(&expression)?);
            if result.should_return() { return Ok(result); }
            let other = result.value.clone().unwrap();

            base = match operator {
                MultiplicativeOperator::Multiply => base.multiply(&other, node.location.clone()),
                MultiplicativeOperator::Divide   => base.divide(&other, node.location.clone()),
            }?;
        }

        result.success(Some(base));
        return Ok(result);
    }

    fn visit_unary_expression(&mut self, node: &UnaryExpression) -> Result<RuntimeResult> {
        return match node {
            UnaryExpression::Operator { location, operator, expression } => {
                let mut result = self.visit_unary_expression(&**expression)?;
                if result.should_return() { return Ok(result); }
                let base = result.value.clone().unwrap();
                let out = match operator {
                    UnaryOperator::Plus  => base,
                    UnaryOperator::Minus => Value::Number(Decimal::ZERO).minus(&base, location.clone())?,
                    UnaryOperator::Not   => Value::Bool(base.is_false()),
                };
                result.success(Some(out));
                return Ok(result);
            },
            UnaryExpression::Power(expression) => self.visit_exponential_expression(expression),
        };
    }

    fn visit_exponential_expression(&mut self, node: &ExponentialExpression) -> Result<RuntimeResult> {
        let mut result = self.visit_atom(&node.base)?;
        if result.should_return() { return Ok(result); }
        let mut base = result.value.clone().unwrap();

        if let Some(exponent) = &node.exponent {
            result.register(self.visit_unary_expression(exponent)?);
            if result.should_return() { return Ok(result); }
            let exponent = result.value.clone().unwrap();

            base = base.power(&exponent, node.location.clone())?;
        }

        result.success(Some(base));
        return Ok(result);
    }

    fn visit_atom(&mut self, node: &Atom) -> Result<RuntimeResult> {
        let mut result = RuntimeResult::new();
        let value =  match node {
            Atom::Number(value) => Value::Number(value.clone()),
            Atom::Bool(value) => Value::Bool(value.clone()),
            Atom::String(value) => Value::String(value.clone()),
            Atom::Null => Value::Null,
            Atom::Identifier { location, name } => self.find_var(name, location.clone())?.clone(),
            Atom::Call(expression) => {
                result.register(self.visit_call_expression(expression)?);
                if result.should_return() { return Ok(result); }
                result.success(result.value.clone());
                return Ok(result);
            },
            Atom::Expression(expression) => {
                result.register(self.visit_expression(expression)?);
                if result.should_return() { return Ok(result); }
                result.success(result.value.clone());
                return Ok(result);
            },
        };
        result.success(Some(value));
        return Ok(result);
    }

    fn visit_call_expression(&mut self, node: &CallExpression) -> Result<RuntimeResult> {
        let mut result = RuntimeResult::new();

        let value = self.find_var(&node.identifier, node.location.clone())?.clone();
        let (args, statements) = match value {
            Value::Function(args, statements) => (args, statements),
            Value::BuiltIn => {
                let mut args: Vec<Value> = vec![];
                for arg in &node.args {
                    result.register(self.visit_expression(&arg)?);
                    if result.should_return() { return Ok(result); }
                    args.push(result.value.clone().unwrap());
                }

                let value = match node.identifier.as_str() {
                    "print" => built_in::print(args),
                    "printl" => built_in::printl(args),
                    "typeOf" => built_in::type_of(args, node.location.clone()),
                    _ => panic!(),
                }?;
                result.success(Some(value));
                return Ok(result);
            },
            _ => error!(TypeError, node.location.clone(), "Type {} is not callable", type_of(&value)),
        };

        if args.len() != node.args.len() {
            error!(
                TypeError,
                node.location.clone(),
                "Function '{}' takes {} argument, however {} were supplied",
                node.identifier,
                args.len(),
                node.args.len(),
            );
        }

        self.push_scope();
        for (index, arg) in args.iter().enumerate() {
            result.register(self.visit_expression(&node.args[index])?);
            if result.should_return() { return Ok(result); }
            self.current_scope().insert(arg.clone(), result.value.clone().unwrap());
        }
        result.register(self.visit_statements(&statements, false)?);

        if result.return_value == None {
            result.success(Some(Value::Void));
        } else {
            result.success(result.return_value.clone());
        }
        return Ok(result);
    }
}
