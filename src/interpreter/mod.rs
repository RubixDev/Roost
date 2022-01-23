mod value;
mod runtime_result;
mod built_in;

use std::collections::HashMap;
use bigdecimal::{Zero, Signed};
use num_bigint::BigInt;
use runtime_result::RuntimeResult;
use value::{Value, types::type_of, types::Type};
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
    Number,
    TernaryExpression,
    CallExpression,
    ExponentialExpression,
};
use self::value::{
    truth::Truth,
    calculative_operations::CalculativeOperations,
    relational_operations::RelationalOperations,
};

pub struct Interpreter {
    start_node: Statements,
    scopes: Vec<HashMap<String, Value>>,
    current_scope_index: usize,
}

impl Interpreter {
    pub fn new(start_node: Statements) -> Interpreter {
        return Interpreter {
            start_node,
            scopes: vec![HashMap::from([
                (String::from("print"), Value::BuiltIn),
                (String::from("printl"), Value::BuiltIn),
                (String::from("answer"), Value::Int(BigInt::from(42))),
            ])],
            current_scope_index: 0
        };
    }

    pub fn run(&mut self) {
        self.visit_statements(&self.start_node.clone(), true);
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

    fn find_var(&self, name: &String) -> &Value {
        let mut scope = self.current_scope_index;
        loop {
            if self.scopes[scope].contains_key(name) {
                return self.scopes[scope].get(name).unwrap();
            }
            if scope == 0 {
                panic!("ReferenceError at position {{}}: Variable or function with name '{}' not found", name);
            }
            scope -= 1;
        }
    }

    fn find_var_scope(&self, name: &String) -> usize {
        let mut scope = self.current_scope_index;
        loop {
            if scope == 0 {
                panic!("ReferenceError at position {{}}: Variable or function with name '{}' not found", name);
            }
            if self.scopes[scope].contains_key(name) {
                return scope;
            }
            scope -= 1;
        }
    }

    // -----------------------------------------------

    fn visit_statements(&mut self, node: &Statements, new_scope: bool) -> RuntimeResult {
        let mut result = RuntimeResult::new();
        if new_scope { self.push_scope(); }
        for statement in &node.statements {
            result.register(self.visit_statement(&statement));
            if result.should_return() { break; }
        }
        self.pop_scope();
        return result;
    }

    fn visit_statement(&mut self, node: &Statement) -> RuntimeResult {
        return match node {
            Statement::Declare   (node) => self.visit_declare_statement(node),
            Statement::Assign    (node) => self.visit_assign_statement(node),
            Statement::If        (node) => self.visit_if_statement(node),
            Statement::Loop      (node) => self.visit_loop_statement(node),
            Statement::While     (node) => self.visit_while_statement(node),
            Statement::For       (node) => self.visit_for_statement(node),
            Statement::Function  (node) => self.visit_function_declaration(node),
            Statement::Expression(node) => self.visit_expression(node),
            Statement::Break            => self.visit_break_statement(),
            Statement::Continue         => self.visit_continue_statement(),
            Statement::Return    (node) => self.visit_return_statement(node),
        };
    }

    fn visit_declare_statement(&mut self, node: &DeclareStatement) -> RuntimeResult {
        let mut result = self.visit_expression(&node.expression);
        if result.should_return() { return result; }

        self.current_scope().insert(node.identifier.clone(), result.value.clone().unwrap());
        result.success(None);
        return result;
    }

    fn visit_assign_statement(&mut self, node: &AssignStatement) -> RuntimeResult {
        let mut result = self.visit_expression(&node.expression);
        if result.should_return() { return result; }
        let new_value = result.value.clone().unwrap();
        let value_scope = self.find_var_scope(&node.identifier);
        let value = self.scopes[value_scope][&node.identifier].clone();

        let old_type = type_of(&value);
        let new_type = type_of(&new_value);
        if new_type != Type::Null && old_type != new_type {
            panic!("TypeError at position {{}}: Expected type '{}', got '{}'", old_type, new_type);
        }

        self.scopes[value_scope].insert(node.identifier.clone(), match node.operator {
            AssignOperator::Normal   => new_value.clone(),
            AssignOperator::Plus     => value.plus(&new_value),
            AssignOperator::Minus    => value.minus(&new_value),
            AssignOperator::Multiply => value.multiply(&new_value),
            AssignOperator::Divide   => value.divide(&new_value),
        });

        result.success(None);
        return result;
    }

    fn visit_if_statement(&mut self, node: &IfStatement) -> RuntimeResult {
        let mut result = self.visit_expression(&node.condition);
        if result.should_return() { return result; }
        let condition = result.value.clone().unwrap();

        if condition.is_true() {
            result.register(self.visit_statements(&node.block, true))
        } else {
            result.register(self.visit_statements(&node.else_block, true))
        }

        if !result.should_return() { result.success(None); }
        return result;
    }

    fn visit_loop_statement(&mut self, node: &LoopStatement) -> RuntimeResult {
        let mut result = RuntimeResult::new();

        loop {
            result.register(self.visit_statements(&node.block, true));
            if result.should_continue { continue; }
            if result.should_break { break; }
            if result.should_return() { return result; }
        }

        result.success(None);
        return result;
    }

    fn visit_while_statement(&mut self, node: &WhileStatement) -> RuntimeResult {
        let mut result = RuntimeResult::new();

        loop {
            result.register(self.visit_expression(&node.condition));
            if result.should_return() { return result; }
            if !result.value.clone().unwrap().is_true() { break; }

            result.register(self.visit_statements(&node.block, true));
            if result.should_continue { continue; }
            if result.should_break { break; }
            if result.should_return() { return result; }
        }

        result.success(None);
        return result;
    }

    fn visit_for_statement(&mut self, node: &ForStatement) -> RuntimeResult {
        let mut result = self.visit_expression(&node.expression);
        if result.should_return() { return result; }
        let expression = result.value.clone().unwrap();

        let range = match expression {
            Value::Range(included, start, end) => {
                let upper = if included { end + 1u8 } else { end };
                (start, upper)
            },
            _ => panic!("TypeError at position {{}}: Cannot iterate through type {}", type_of(&expression)),
        };
        let backwards = &range.1 - 1u8 <= range.0;
        let iterations = (&range.0 - range.1).abs();
        let mut i = BigInt::zero();
        while i < iterations {
            self.push_scope();
            self.current_scope().insert(node.identifier.clone(), Value::Int(
                if backwards { &iterations - &i } else { &i + 0 } + &range.0
            ));

            result.register(self.visit_statements(&node.block, false));
            if result.should_continue { continue; }
            if result.should_break { break; }
            if result.should_return() { return result; }

            i += 1u8;
        }

        result.success(None);
        return result;
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

    fn visit_return_statement(&mut self, node: &ReturnStatement) -> RuntimeResult {
        let mut result = RuntimeResult::new();
        if let Some(expression) = &node.expression {
            result.register(self.visit_expression(&expression));
            if result.should_return() { return result; }
            result.success_return(result.value.clone());
        } else {
            result.success_return(Some(Value::Void));
        }
        return result;
    }


    fn visit_expression(&mut self, node: &Expression) -> RuntimeResult {
        let mut result = self.visit_ternary_expression(&*node.base);
        if result.should_return() { return result; }
        let val1 = result.value.clone().unwrap();

        if let Some((inclusive, expression)) = &*node.range {
            result.register(self.visit_ternary_expression(&expression));
            if result.should_return() { return result; }
            let val2 = result.value.clone().unwrap();

            let range = match val1 {
                Value::Int(start) => match val2 {
                    Value::Int(end) => Value::Range(*inclusive, start, end),
                    _ => panic!("TypeError at position {{}}: Range bounds have to be of type int, got {}", type_of(&val2)),
                },
                _ => panic!("TypeError at position {{}}: Range bounds have to be of type int, got {}", type_of(&val1)),
            };
            result.success(Some(range));
        } else {
            result.success(Some(val1));
        }

        return result;
    }

    fn visit_ternary_expression(&mut self, node: &TernaryExpression) -> RuntimeResult {
        let mut result = self.visit_or_expression(&node.base);
        if result.should_return() { return result; }
        let condition = result.value.clone().unwrap();

        if let Some((if_expr, else_expr)) = &node.ternary {
            if condition.is_true() {
                result.register(self.visit_expression(&if_expr));
            } else {
                result.register(self.visit_expression(&else_expr));
            }
            if result.should_return() { return result; }
            result.success(result.value.clone());
        } else {
            result.success(Some(condition));
        }

        return result;
    }

    fn visit_or_expression(&mut self, node: &OrExpression) -> RuntimeResult {
        let mut result = self.visit_and_expression(&node.base);
        if result.should_return() { return result; }
        let base = result.value.clone().unwrap();

        if !node.following.is_empty() {
            if base.is_true() {
                result.success(Some(Value::Bool(true)));
                return result;
            }
            for expression in &node.following {
                result.register(self.visit_and_expression(&expression));
                if result.should_return() { return result; }
                if result.value.clone().unwrap().is_true() {
                    result.success(Some(Value::Bool(true)));
                    return result;
                }
            }
            result.success(Some(Value::Bool(false)));
        } else {
            result.success(Some(base));
        }

        return result;
    }

    fn visit_and_expression(&mut self, node: &AndExpression) -> RuntimeResult {
        let mut result = self.visit_equality_expression(&node.base);
        if result.should_return() { return result; }
        let base = result.value.clone().unwrap();

        if !node.following.is_empty() {
            if base.is_false() {
                result.success(Some(Value::Bool(false)));
                return result;
            }
            for expression in &node.following {
                result.register(self.visit_equality_expression(&expression));
                if result.should_return() { return result; }
                if result.value.clone().unwrap().is_false() {
                    result.success(Some(Value::Bool(false)));
                    return result;
                }
            }
            result.success(Some(Value::Bool(true)));
        } else {
            result.success(Some(base));
        }

        return result;
    }

    fn visit_equality_expression(&mut self, node: &EqualityExpression) -> RuntimeResult {
        let mut result = self.visit_relational_expression(&node.base);
        if result.should_return() { return result; }
        let base = result.value.clone().unwrap();

        if let Some((operator, expression)) = &node.other {
            result.register(self.visit_relational_expression(&expression));
            if result.should_return() { return result; }
            let equal = base == result.value.clone().unwrap();
            let out = match operator {
                EqualityOperator::Equal    =>  equal,
                EqualityOperator::NotEqual => !equal,
            };
            result.success(Some(Value::Bool(out)));
        } else {
            result.success(Some(base));
        }

        return result;
    }

    fn visit_relational_expression(&mut self, node: &RelationalExpression) -> RuntimeResult {
        let mut result = self.visit_additive_expression(&node.base);
        if result.should_return() { return result; }
        let base = result.value.clone().unwrap();

        if let Some((operator, expression)) = &node.other {
            result.register(self.visit_additive_expression(&expression));
            if result.should_return() { return result; }
            let other = result.value.clone().unwrap();

            let out = match operator {
                RelationalOperator::LessThan           => base.less_than(&other),
                RelationalOperator::GreaterThan        => base.greater_than(&other),
                RelationalOperator::LessThanOrEqual    => base.less_than_or_equal(&other),
                RelationalOperator::GreaterThanOrEqual => base.greater_than_or_equal(&other),
            };
            result.success(Some(out));
        } else {
            result.success(Some(base));
        }

        return result;
    }

    fn visit_additive_expression(&mut self, node: &AdditiveExpression) -> RuntimeResult {
        let mut result = self.visit_multiplicative_expression(&node.base);
        if result.should_return() { return result; }
        let mut base = result.value.clone().unwrap();

        for (operator, expression) in &node.following {
            result.register(self.visit_multiplicative_expression(&expression));
            if result.should_return() { return result; }
            let other = result.value.clone().unwrap();

            base = match operator {
                AdditiveOperator::Plus  => base.plus(&other),
                AdditiveOperator::Minus => base.minus(&other),
            };
        }

        result.success(Some(base));
        return result;
    }

    fn visit_multiplicative_expression(&mut self, node: &MultiplicativeExpression) -> RuntimeResult {
        let mut result = self.visit_exponential_expression(&node.base);
        if result.should_return() { return result; }
        let mut base = result.value.clone().unwrap();

        for (operator, expression) in &node.following {
            result.register(self.visit_exponential_expression(&expression));
            if result.should_return() { return result; }
            let other = result.value.clone().unwrap();

            base = match operator {
                MultiplicativeOperator::Multiply => base.multiply(&other),
                MultiplicativeOperator::Divide   => base.divide(&other),
            };
        }

        result.success(Some(base));
        return result;
    }

    fn visit_exponential_expression(&mut self, node: &ExponentialExpression) -> RuntimeResult {
        let mut result = self.visit_unary_expression(&node.base);
        if result.should_return() { return result; }
        let mut base = result.value.clone().unwrap();

        if !node.following.is_empty() {
            result.register(self.visit_unary_expression(node.following.last().unwrap()));
            if result.should_return() { return result; }
            let mut exponent = result.value.clone().unwrap();
            let mut index = (node.following.len() as isize) - 2;

            while index != -1 {
                result.register(self.visit_unary_expression(&node.following[index as usize]));
                if result.should_return() { return result; }
                let base = result.value.clone().unwrap();
                exponent = base.power(&exponent);

                index -= 1;
            }

            base = base.power(&exponent);
        }

        result.success(Some(base));
        return result;
    }

    fn visit_unary_expression(&mut self, node: &UnaryExpression) -> RuntimeResult {
        return match node {
            UnaryExpression::Operator(operator, expression) => {
                let mut result = self.visit_unary_expression(&**expression);
                if result.should_return() { return result; }
                let base = result.value.clone().unwrap();
                let out = match operator {
                    UnaryOperator::Plus  => base,
                    UnaryOperator::Minus => Value::Int(BigInt::zero()).minus(&base),
                    UnaryOperator::Not   => Value::Bool(base.is_false()),
                };
                result.success(Some(out));
                return result;
            },
            UnaryExpression::Expression(expression) => self.visit_expression(expression),
            UnaryExpression::Atom(atom) => self.visit_atom(atom),
        };
    }

    fn visit_atom(&mut self, node: &Atom) -> RuntimeResult {
        let mut result = RuntimeResult::new();
        let value =  match node {
            Atom::Number(value) => match value {
                Number::Int(value)   => Value::Int(value.clone()),
                Number::Float(value) => Value::Float(value.clone()),
            },
            Atom::Bool(value) => Value::Bool(value.clone()),
            Atom::String(value) => Value::String(value.clone()),
            Atom::Null => Value::Null,
            Atom::Identifier(name) => self.find_var(name).clone(),
            Atom::Call(expression) => {
                result.register(self.visit_call_expression(expression));
                if result.should_return() { return result; }
                result.success(result.value.clone());
                return result;
            },
        };
        result.success(Some(value));
        return result;
    }

    fn visit_call_expression(&mut self, node: &CallExpression) -> RuntimeResult {
        let mut result = RuntimeResult::new();

        let value = self.find_var(&node.identifier).clone();
        let (args, statements) = match value {
            Value::Function(args, statements) => (args, statements),
            Value::BuiltIn => {
                let mut args: Vec<Value> = vec![];
                for arg in &node.args {
                    result.register(self.visit_expression(&arg));
                    if result.should_return() { return result; }
                    args.push(result.value.clone().unwrap());
                }

                let value = match node.identifier.as_str() {
                    "print" => built_in::print(args),
                    "printl" => built_in::printl(args),
                    _ => panic!(),
                };
                result.success(Some(value));
                return result;
            },
            _ => panic!("TypeError at position {{}}: Type {} is not callable", type_of(&value)),
        };

        if args.len() != node.args.len() {
            panic!(
                "TypeError at position {{}}: Function '{}' takes {} argument, however {} were supplied",
                node.identifier,
                args.len(),
                node.args.len(),
            );
        }

        self.push_scope();
        for (index, arg) in args.iter().enumerate() {
            result.register(self.visit_expression(&node.args[index]));
            if result.should_return() { return result; }
            self.current_scope().insert(arg.clone(), result.value.clone().unwrap());
        }
        result.register(self.visit_statements(&statements, false));

        if result.return_value == None {
            result.success(Some(Value::Void));
        } else {
            result.success(result.return_value.clone());
        }
        return result;
    }
}
