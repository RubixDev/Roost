pub mod value;
mod runtime_result;
mod built_in;

use std::collections::HashMap;
#[cfg(not(feature = "no_std_io"))]
use std::io::Write;
#[cfg(feature = "no_std_io")]
use crate::io::Write;
use runtime_result::RuntimeResult;
use rust_decimal::Decimal;
use value::{Value, types::type_of, types::Type};
use crate::error::{Result, Location};
use crate::nodes::{
    Statements,
    Statement,
    DeclareStatement,
    AssignStatement,
    IfExpression,
    LoopStatement,
    WhileStatement,
    ForStatement,
    FunctionDeclaration,
    ReturnStatement,
    Expression,
    OrExpression,
    AndExpression,
    EqualityExpression,
    RelationalExpression,
    AdditiveExpression,
    MultiplicativeExpression,
    UnaryExpression,
    Atom,
    CallExpression,
    ExponentialExpression,
    FunExpression,
    MemberExpression,
    CallPart,
    MemberPart,
    ClassDeclaration,
    ClassExpression,
};
use crate::tokens::TokenType;
use self::value::{BuiltIn, SpecialBuiltIn};
use self::value::members::Members;
use self::value::{
    truth::Truth,
    calculative_operations::CalculativeOperations,
    relational_operations::RelationalOperations, iterator::ToIterator,
};

macro_rules! should_return {
    ($result:ident) => {
        if $result.should_return() { return Ok($result); }
    };
}

macro_rules! expr_val {
    ($result:ident, $run_res:expr) => {
        $result.register($run_res?);
        should_return!($result);
        $result.success($result.value.clone());
        return Ok($result)
    };
}

macro_rules! current_scope {
    ($self:ident) => {
        $self.scopes[$self.current_scope_index]
    };
}

macro_rules! register {
    ($result:ident, $new_res:expr) => {{
        $result.register($new_res);
        should_return!($result);
        $result.value.clone().unwrap_or(Value::Null)
    }};
}

pub trait Exit {
    fn exit(&mut self, code: i32);
}

pub struct Interpreter<OUT, EXIT> where OUT: Write, EXIT: Exit {
    start_node: Statements,
    pub scopes: Vec<HashMap<String, Value>>,
    pub current_scope_index: usize,
    stdout: OUT,
    exit: EXIT,
}

impl <OUT: Write, EXIT: Exit> Interpreter<OUT, EXIT> {
    pub fn new(start_node: Statements, stdout: OUT, exit: EXIT) -> Self {
        return Interpreter {
            start_node,
            scopes: vec![HashMap::from([
                (String::from("print"), Value::BuiltIn(BuiltIn::Special(SpecialBuiltIn::Print(false)))),
                (String::from("printl"), Value::BuiltIn(BuiltIn::Special(SpecialBuiltIn::Print(true)))),
                (String::from("typeOf"), Value::BuiltIn(BuiltIn::Function(built_in::type_of))),
                (String::from("exit"), Value::BuiltIn(BuiltIn::Special(SpecialBuiltIn::Exit))),
                (String::from("answer"), Value::Number(Decimal::from(42))),
            ])],
            current_scope_index: 0,
            stdout,
            exit,
        };
    }

    pub fn new_run(start_node: Statements, stdout: OUT, exit: EXIT) -> Result<RuntimeResult> {
        let mut interpreter = Self::new(start_node, stdout, exit);
        return interpreter.run(true);
    }

    pub fn run(&mut self, new_scope: bool) -> Result<RuntimeResult> {
        return self.visit_statements(&self.start_node.clone(), new_scope);
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.current_scope_index += 1;
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
        self.current_scope_index -= 1;
    }

    fn find_var(&self, name: &String, start_loc: Location, end_loc: Location) -> Result<&Value> {
        let mut scope = self.current_scope_index;
        loop {
            if self.scopes[scope].contains_key(name) {
                return Ok(self.scopes[scope].get(name).unwrap());
            }
            if scope == 0 {
                error!(ReferenceError, start_loc, end_loc, "Variable or function with name '{}' not found", name);
            }
            scope -= 1;
        }
    }

    fn find_var_scope(&self, name: &String, start_loc: Location, end_loc: Location) -> Result<usize> {
        let mut scope = self.current_scope_index;
        loop {
            if scope == 0 {
                error!(ReferenceError, start_loc, end_loc, "Variable or function with name '{}' not found", name);
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
        if new_scope { self.pop_scope(); }
        if result.value == None && !result.should_return() { result.success(Some(Value::Null)) }
        return Ok(result);
    }

    fn visit_statement(&mut self, node: &Statement) -> Result<RuntimeResult> {
        return match node {
            Statement::Declare   (node) => self.visit_declare_statement(node),
            Statement::Assign    (node) => self.visit_assign_statement(node),
            Statement::Loop      (node) => self.visit_loop_statement(node),
            Statement::While     (node) => self.visit_while_statement(node),
            Statement::For       (node) => self.visit_for_statement(node),
            Statement::Function  (node) => Ok(self.visit_function_declaration(node)),
            Statement::Class     (node) => self.visit_class_declaration(node),
            Statement::Expression(node) => self.visit_expression(node),
            Statement::Break            => Ok(self.visit_break_statement()),
            Statement::Continue         => Ok(self.visit_continue_statement()),
            Statement::Return    (node) => self.visit_return_statement(node),
        };
    }

    fn visit_declare_statement(&mut self, node: &DeclareStatement) -> Result<RuntimeResult> {
        let mut result = RuntimeResult::new();
        if let Some(expr) = &node.expression {
            register!(result, self.visit_expression(expr)?);
        } else {
            result.success(Some(Value::Null));
        }

        current_scope!(self).insert(node.identifier.clone(), result.value.clone().unwrap());
        result.success(None);
        return Ok(result);
    }

    fn visit_assign_statement(&mut self, node: &AssignStatement) -> Result<RuntimeResult> {
        let mut result = self.visit_expression(&node.expression)?;
        should_return!(result);
        let new_value = result.value.clone().unwrap();
        let value_scope = self.find_var_scope(&node.identifier, node.start.clone(), node.end.clone())?;
        let value = self.scopes[value_scope][&node.identifier].clone();

        let new_val = register!(result, self.assign_member(
            value,
            &node.parts,
            &node.operator,
            new_value,
            node.start.clone(),
            node.end.clone()
        )?);
        self.scopes[value_scope].insert(node.identifier.clone(), new_val);

        result.success(None);
        return Ok(result);
    }

    fn assign_member(&mut self, base: Value, parts: &Vec<MemberPart>, operator: &TokenType, rhs: Value, start_loc: Location, end_loc: Location) -> Result<RuntimeResult> {
        let mut result = RuntimeResult::new();
        if parts.is_empty() {
            let old_type = type_of(&base);
            let new_type = type_of(&rhs);
            if old_type != Type::Null && new_type != Type::Null && old_type != new_type {
                error!(TypeError, start_loc, end_loc, "Cannot assign type '{}' to '{}'", new_type, old_type);
            }
            result.success(Some(match operator {
                TokenType::Assign          => Ok(rhs.clone()),
                TokenType::PlusAssign      => base.plus      (&rhs, start_loc, end_loc),
                TokenType::MinusAssign     => base.minus     (&rhs, start_loc, end_loc),
                TokenType::MultiplyAssign  => base.multiply  (&rhs, start_loc, end_loc),
                TokenType::DivideAssign    => base.divide    (&rhs, start_loc, end_loc),
                TokenType::ModuloAssign    => base.modulo    (&rhs, start_loc, end_loc),
                TokenType::IntDivideAssign => base.int_divide(&rhs, start_loc, end_loc),
                TokenType::PowerAssign     => base.power     (&rhs, start_loc, end_loc),
                _ => panic!(),
            }?));
        } else {
            let next_base = match &parts[0] {
                MemberPart::Identifier(name) => base.get_member(name, start_loc.clone(), end_loc.clone()),
            }?;
            let next_parts = Vec::from(&parts[1..]);
            let new_val = register!(result, self.assign_member(next_base, &next_parts, operator, rhs, start_loc.clone(), end_loc.clone())?);
            result.success(Some(match &parts[0] {
                MemberPart::Identifier(name) => base.set_member(name, new_val, start_loc, end_loc),
            }?));
        }

        return Ok(result);
    }

    fn visit_if_expression(&mut self, node: &IfExpression) -> Result<RuntimeResult> {
        let mut result = self.visit_expression(&node.condition)?;
        should_return!(result);
        let condition = result.value.clone().unwrap();

        if condition.is_true() {
            result.register(self.visit_statements(&node.block, true)?);
        } else {
            if let Some(block) = &node.else_block {
                result.register(self.visit_statements(block, true)?);
            } else {
                result.success(Some(Value::Null));
            }
        }

        return Ok(result);
    }

    fn visit_loop_statement(&mut self, node: &LoopStatement) -> Result<RuntimeResult> {
        let mut result = RuntimeResult::new();

        loop {
            result.register(self.visit_statements(&node.block, true)?);
            if result.should_continue { continue; }
            if result.should_break { break; }
            should_return!(result);
        }

        result.success(None);
        return Ok(result);
    }

    fn visit_while_statement(&mut self, node: &WhileStatement) -> Result<RuntimeResult> {
        let mut result = RuntimeResult::new();

        loop {
            let res = register!(result, self.visit_expression(&node.condition)?);
            if !res.is_true() { break; }

            result.register(self.visit_statements(&node.block, true)?);
            if result.should_continue { continue; }
            if result.should_break { break; }
            should_return!(result);
        }

        result.success(None);
        return Ok(result);
    }

    fn visit_for_statement(&mut self, node: &ForStatement) -> Result<RuntimeResult> {
        let mut result = self.visit_expression(&node.expression)?;
        should_return!(result);
        let expression = result.value.clone().unwrap();

        let mut iter = expression.to_iter(node.start.clone(), node.end.clone())?;
        while let Some(i) = iter.next() {
            self.push_scope();
            current_scope!(self).insert(node.identifier.clone(), i.clone());

            result.register(self.visit_statements(&node.block, false)?);
            self.pop_scope();
            if result.should_continue { continue; }
            if result.should_break { break; }
            should_return!(result);
        }

        result.success(None);
        return Ok(result);
    }

    fn visit_function_declaration(&mut self, node: &FunctionDeclaration) -> RuntimeResult {
        current_scope!(self).insert(node.identifier.clone(), Value::Function(node.is_static, node.params.clone(), node.block.clone()));
        return RuntimeResult::new();
    }

    fn visit_class_declaration(&mut self, node: &ClassDeclaration) -> Result<RuntimeResult> {
        let mut result = RuntimeResult::new();
        self.push_scope();
        for declaration in &node.block.statements {
            register!(result, self.visit_statement(declaration)?);
        }
        let members = current_scope!(self).clone();
        self.pop_scope();

        current_scope!(self).insert(node.identifier.clone(), Value::Class(members));
        return Ok(result);
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
            let val = register!(result, self.visit_expression(&expression)?);
            result.success_return(Some(val));
        } else {
            result.success_return(Some(Value::Null));
        }
        return Ok(result);
    }


    fn visit_expression(&mut self, node: &Expression) -> Result<RuntimeResult> {
        let mut result = self.visit_or_expression(&*node.base)?;
        should_return!(result);
        let val1 = result.value.clone().unwrap();

        if let Some((inclusive, expression)) = &*node.range {
            let val2 = register!(result, self.visit_or_expression(&expression)?);

            let range = match val1 {
                Value::Number(start) => match val2 {
                    Value::Number(end) => {
                        if !start.fract().is_zero() || !end.fract().is_zero() {
                            error!(ValueError, node.start.clone(), node.end.clone(), "Range bounds have to be integers");
                        }
                        if !inclusive && start != end {
                            if start > end {
                                Value::Range(start, end + Decimal::ONE)
                            } else {
                                Value::Range(start, end - Decimal::ONE)
                            }
                        } else { Value::Range(start, end) }
                    },
                    _ => error!(TypeError, node.start.clone(), node.end.clone(), "Range bounds have to be of type number, got {}", type_of(&val1)),
                },
                _ => error!(TypeError, node.start.clone(), node.end.clone(), "Range bounds have to be of type number, got {}", type_of(&val1)),
            };
            result.success(Some(range));
        } else {
            result.success(Some(val1));
        }

        return Ok(result);
    }

    fn visit_or_expression(&mut self, node: &OrExpression) -> Result<RuntimeResult> {
        let mut result = self.visit_and_expression(&node.base)?;
        should_return!(result);
        let base = result.value.clone().unwrap();

        if !node.following.is_empty() {
            if base.is_true() {
                result.success(Some(Value::Bool(true)));
                return Ok(result);
            }
            for expression in &node.following {
                let res = register!(result, self.visit_and_expression(&expression)?);
                if res.is_true() {
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
        should_return!(result);
        let base = result.value.clone().unwrap();

        if !node.following.is_empty() {
            if base.is_false() {
                result.success(Some(Value::Bool(false)));
                return Ok(result);
            }
            for expression in &node.following {
                let res = register!(result, self.visit_equality_expression(&expression)?);
                if res.is_false() {
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
        should_return!(result);
        let base = result.value.clone().unwrap();

        if let Some((operator, expression)) = &node.other {
            let res = register!(result, self.visit_relational_expression(&expression)?);
            let equal = base == res;
            let out = match operator {
                TokenType::Equal    =>  equal,
                TokenType::NotEqual => !equal,
                _ => panic!(),
            };
            result.success(Some(Value::Bool(out)));
        } else {
            result.success(Some(base));
        }

        return Ok(result);
    }

    fn visit_relational_expression(&mut self, node: &RelationalExpression) -> Result<RuntimeResult> {
        let mut result = self.visit_additive_expression(&node.base)?;
        should_return!(result);
        let base = result.value.clone().unwrap();

        if let Some((operator, expression)) = &node.other {
            let other = register!(result, self.visit_additive_expression(&expression)?);

            let out = match operator {
                TokenType::LessThan           => base.less_than(&other, node.start.clone(), node.end.clone()),
                TokenType::GreaterThan        => base.greater_than(&other, node.start.clone(), node.end.clone()),
                TokenType::LessThanOrEqual    => base.less_than_or_equal(&other, node.start.clone(), node.end.clone()),
                TokenType::GreaterThanOrEqual => base.greater_than_or_equal(&other, node.start.clone(), node.end.clone()),
                _ => panic!(),
            }?;
            result.success(Some(out));
        } else {
            result.success(Some(base));
        }

        return Ok(result);
    }

    fn visit_additive_expression(&mut self, node: &AdditiveExpression) -> Result<RuntimeResult> {
        let mut result = self.visit_multiplicative_expression(&node.base)?;
        should_return!(result);
        let mut base = result.value.clone().unwrap();

        for (operator, expression) in &node.following {
            let other = register!(result, self.visit_multiplicative_expression(&expression)?);

            base = match operator {
                TokenType::Plus  => base.plus(&other, node.start.clone(), node.end.clone()),
                TokenType::Minus => base.minus(&other, node.start.clone(), node.end.clone()),
                _ => panic!(),
            }?;
        }

        result.success(Some(base));
        return Ok(result);
    }

    fn visit_multiplicative_expression(&mut self, node: &MultiplicativeExpression) -> Result<RuntimeResult> {
        let mut result = self.visit_unary_expression(&node.base)?;
        should_return!(result);
        let mut base = result.value.clone().unwrap();

        for (operator, expression) in &node.following {
            let other = register!(result, self.visit_unary_expression(&expression)?);

            base = match operator {
                TokenType::Multiply  => base.multiply(&other, node.start.clone(), node.end.clone()),
                TokenType::Divide    => base.divide(&other, node.start.clone(), node.end.clone()),
                TokenType::Modulo    => base.modulo(&other, node.start.clone(), node.end.clone()),
                TokenType::IntDivide => base.int_divide(&other, node.start.clone(), node.end.clone()),
                _ => panic!(),
            }?;
        }

        result.success(Some(base));
        return Ok(result);
    }

    fn visit_unary_expression(&mut self, node: &UnaryExpression) -> Result<RuntimeResult> {
        return match node {
            UnaryExpression::Operator { start, end, operator, expression } => {
                let mut result = self.visit_unary_expression(&**expression)?;
                should_return!(result);
                let base = result.value.clone().unwrap();
                let out = match operator {
                    TokenType::Plus  => base,
                    TokenType::Minus => Value::Number(Decimal::ZERO).minus(&base, start.clone(), end.clone())?,
                    TokenType::Not   => Value::Bool(base.is_false()),
                    _ => panic!(),
                };
                result.success(Some(out));
                return Ok(result);
            },
            UnaryExpression::Power(expression) => self.visit_exponential_expression(expression),
        };
    }

    fn visit_exponential_expression(&mut self, node: &ExponentialExpression) -> Result<RuntimeResult> {
        let mut result = self.visit_call_expression(&node.base)?;
        should_return!(result);
        let mut base = result.value.clone().unwrap();

        if let Some(exponent) = &node.exponent {
            let exponent = register!(result, self.visit_unary_expression(exponent)?);

            base = base.power(&exponent, node.start.clone(), node.end.clone())?;
        }

        result.success(Some(base));
        return Ok(result);
    }

    fn call_value(&mut self, result: &RuntimeResult, call_args: &Vec<Expression>, start_loc: Location, end_loc: Location) -> Result<RuntimeResult> {
        let value = result.value.clone().unwrap();
        let parent = result.parent_value.clone();
        let mut result = RuntimeResult::new();

        match value {
            Value::Function(_, args, statements) => {
                if args.len() != call_args.len() {
                    error!(
                        TypeError,
                        start_loc,
                        end_loc,
                        "Function takes {} arguments, however {} were supplied",
                        args.len(),
                        call_args.len(),
                    );
                }

                self.push_scope();
                current_scope!(self).insert(String::from("this"), parent.unwrap());
                for (index, arg) in args.iter().enumerate() {
                    let val = register!(result, self.visit_expression(&call_args[index])?);
                    current_scope!(self).insert(arg.clone(), val);
                }
                result.register(self.visit_statements(&statements, false)?);
                self.pop_scope();

                if result.return_value == None {
                    if result.value != None {
                        result.success(result.value.clone());
                    } else {
                        result.success(Some(Value::Null));
                    }
                } else {
                    result.success(result.return_value.clone());
                }
                return Ok(result);
            },
            Value::BuiltIn(built_in) => {
                let mut args: Vec<Value> = vec![];
                for arg in call_args {
                    args.push(register!(result, self.visit_expression(&arg)?));
                }

                let value = match built_in {
                    BuiltIn::Special(fun) => match fun {
                        SpecialBuiltIn::Print(newline) => built_in::print(args, &mut self.stdout, start_loc, end_loc, newline),
                        SpecialBuiltIn::Exit => built_in::exit(args, &mut self.exit, start_loc, end_loc),
                    },
                    BuiltIn::Function(fun) => fun(args, start_loc, end_loc),
                    BuiltIn::Method(fun) => fun(parent.unwrap(), args, start_loc, end_loc),
                }?;
                result.success(Some(value));
                return Ok(result);
            },
            Value::Class(members) => {
                if call_args.len() != 0 {
                    error!(
                        TypeError,
                        start_loc,
                        end_loc,
                        "Class constructor takes no arguments, however {} were supplied",
                        call_args.len(),
                    );
                }
                result.success(Some(Value::Object(members)));
                return Ok(result);
            }
            _ => error!(TypeError, start_loc, end_loc, "Type {} is not callable", type_of(&value)),
        }
    }

    fn visit_call_part(&mut self, result: &mut RuntimeResult, part: &CallPart, start_loc: Location, end_loc: Location) -> Result<()> {
        match part {
            CallPart::Member(part) => self.visit_member_part(result, part, start_loc, end_loc)?,
            CallPart::Arguments(args) => {
                result.register(self.call_value(&result, args, start_loc, end_loc)?);
            },
        }
        return Ok(());
    }

    fn visit_member_part(&mut self, result: &mut RuntimeResult, part: &MemberPart, start_loc: Location, end_loc: Location) -> Result<()> {
        match part {
            MemberPart::Identifier(name) => result.success(Some(result.value.clone().unwrap().get_member(name, start_loc, end_loc)?)),
        }
        return Ok(());
    }

    fn visit_call_expression(&mut self, node: &CallExpression) -> Result<RuntimeResult> {
        let mut result = RuntimeResult::new();

        register!(result, self.visit_member_expression(&node.base)?);

        if node.call == None { return Ok(result); }
        let node_call = node.call.clone().unwrap();

        result.register(self.call_value(
            &result,
            &node_call.clone().0,
            node.start.clone(),
            node.end.clone()
        )?);
        should_return!(result);

        for part in &node_call.1 {
            self.visit_call_part(&mut result, part, node.start.clone(), node.end.clone())?;
        }

        return Ok(result);
    }

    fn visit_member_expression(&mut self, node: &MemberExpression) -> Result<RuntimeResult> {
        let mut result = RuntimeResult::new();

        let value = register!(result, self.visit_atom(&node.base)?);
        result.success(Some(value));

        for part in &node.parts {
            self.visit_member_part(&mut result, part, node.start.clone(), node.end.clone())?;
        }

        return Ok(result);
    }

    fn visit_atom(&mut self, node: &Atom) -> Result<RuntimeResult> {
        let mut result = RuntimeResult::new();
        let value =  match node {
            Atom::Number(value) => Value::Number(value.clone()),
            Atom::Bool(value) => Value::Bool(value.clone()),
            Atom::String(value) => Value::String(value.clone()),
            Atom::Null => Value::Null,
            Atom::Identifier { start, end, name } => self.find_var(name, start.clone(), end.clone())?.clone(),
            Atom::If(expression) => { expr_val!(result, self.visit_if_expression(expression)); },
            Atom::Fun(expression) => { expr_val!(result, self.visit_fun_expression(expression)); },
            Atom::Class(expression) => { expr_val!(result, self.visit_class_expression(expression)); },
            Atom::Expression(expression) => { expr_val!(result, self.visit_expression(expression)); },
            Atom::Block(expression) => { expr_val!(result, self.visit_statements(expression, true)); }
        };
        result.success(Some(value));
        return Ok(result);
    }

    fn visit_fun_expression(&mut self, node: &FunExpression) -> Result<RuntimeResult> {
        let mut result = RuntimeResult::new();
        result.success(Some(Value::Function(true, node.params.clone(), node.block.clone())));
        return Ok(result);
    }

    fn visit_class_expression(&mut self, node: &ClassExpression) -> Result<RuntimeResult> {
        let mut result = RuntimeResult::new();
        self.push_scope();
        for declaration in &node.block.statements {
            register!(result, self.visit_statement(declaration)?);
        }
        let members = current_scope!(self).clone();
        self.pop_scope();

        result.success(Some(Value::Class(members)));
        return Ok(result);
    }
}
