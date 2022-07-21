mod built_in;
mod runtime_result;
pub mod value;

use rust_decimal::{prelude::ToPrimitive, Decimal};

#[cfg(feature = "no_std_io")]
use crate::io::Write;
use crate::{
    error::{Location, Result},
    nodes::*,
    tokens::TokenKind,
};
#[cfg(not(feature = "no_std_io"))]
use std::io::Write;
use std::{collections::HashMap, rc::Rc};

use self::{
    runtime_result::RuntimeResult,
    value::{
        types::{self, Type},
        BuiltIn, Value, WrappedValue,
    },
};

macro_rules! try_visit {
    ($call:expr) => {{
        let res: RuntimeResult = $call;
        if res.should_return() {
            return Ok(res);
        }
        res.take_value()
    }};
}

macro_rules! simple_expr {
    ($name:ident: $type:ident, $next:ident; $($tt:tt)+) => {
        fn $name(&mut self, node: &'tree $type) -> Result<RuntimeResult<'tree>> {
            let mut base = try_visit!(self.$next(&node.base)?);
            simple_expr!(@kind self, node, base, $next, $($tt)+);
            Ok(RuntimeResult::new(Some(base)))
        }
    };
    (@kind $self:ident, $node:ident, $base:ident, $next:ident, $method:ident) => {
        for other in &$node.following {
            let other = try_visit!($self.$next(other)?);
            let out = $base
                .borrow()
                .$method(&other.borrow(), &$node.start, &$node.end)?
                .wrapped();
            $base = out;
        }
    };
    (@kind $self:ident, $node:ident, $base:ident, $next:ident, $($tok:ident => $method:ident),+ $(,)?) => {
        for (tok, other) in &$node.following {
            let other = try_visit!($self.$next(other)?);
            let out = match tok {
                $(TokenKind::$tok => $base.borrow().$method(&other.borrow(), &$node.start, &$node.end),)+
                _ => unreachable!(),
            }?
            .wrapped();
            $base = out;
        }
    };
}

pub struct Interpreter<'tree, O: Write, E: FnOnce(i32)> {
    program: &'tree Program,
    pub scopes: Vec<HashMap<&'tree str, WrappedValue<'tree>>>,
    pub scope_idx: usize,
    stdout: O,
    exit_callback: Option<E>,
}

impl<'tree, O: Write, E: FnOnce(i32)> Interpreter<'tree, O, E> {
    pub fn new(program: &'tree Program, stdout: O, exit_callback: E) -> Self {
        Self {
            program,
            scopes: vec![HashMap::from([
                ("print", Value::BuiltIn(BuiltIn::Print(false)).wrapped()),
                ("println", Value::BuiltIn(BuiltIn::Print(true)).wrapped()),
                (
                    "typeOf",
                    Value::BuiltIn(BuiltIn::Function(built_in::type_of)).wrapped(),
                ),
                (
                    "assert",
                    Value::BuiltIn(BuiltIn::Function(built_in::assert)).wrapped(),
                ),
                (
                    "throw",
                    Value::BuiltIn(BuiltIn::Function(built_in::throw)).wrapped(),
                ),
                ("exit", Value::BuiltIn(BuiltIn::Exit).wrapped()),
                ("answer", Value::Number(42.into()).wrapped()),
            ])],
            scope_idx: 0,
            stdout,
            exit_callback: Some(exit_callback),
        }
    }

    pub fn new_run(
        program: &'tree Program,
        stdout: O,
        exit_callback: E,
    ) -> Result<RuntimeResult<'tree>> {
        Self::new(program, stdout, exit_callback).run(true)
    }

    #[inline]
    pub fn run(&mut self, new_scope: bool) -> Result<RuntimeResult<'tree>> {
        self.visit_program(self.program, new_scope)
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.scope_idx += 1;
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
        self.scope_idx -= 1;
    }

    fn add_var(&mut self, name: &'tree str, value: WrappedValue<'tree>) {
        self.scopes.last_mut().unwrap().insert(name, value);
    }

    fn get_var(
        &self,
        name: &str,
        (start, end): (&Location, &Location),
    ) -> Result<(&WrappedValue<'tree>, usize)> {
        for (idx, scope) in self.scopes.iter().enumerate().rev() {
            if let Some(var) = scope.get(name) {
                return Ok((var, idx));
            }
        }
        error!(
            ReferenceError,
            *start, *end, "Variable with name '{}' not found", name
        );
    }

    ////////////////////////////////////////

    #[inline]
    fn visit_program(
        &mut self,
        node: &'tree Program,
        new_scope: bool,
    ) -> Result<RuntimeResult<'tree>> {
        self.visit_statements(node, new_scope)
    }

    fn visit_statements(
        &mut self,
        node: &'tree Statements,
        new_scope: bool,
    ) -> Result<RuntimeResult<'tree>> {
        if new_scope {
            self.push_scope();
        }
        let mut result = RuntimeResult::new(None);
        for stmt in &node.stmts {
            result = self.visit_statement(stmt)?;
            if result.should_return() {
                break;
            }
        }
        if new_scope {
            self.pop_scope();
        }
        Ok(if result.value.is_none() && !result.should_return() {
            RuntimeResult::new(Some(Value::Null.wrapped()))
        } else {
            result
        })
    }

    #[inline]
    fn visit_block(&mut self, node: &'tree Block, new_scope: bool) -> Result<RuntimeResult<'tree>> {
        self.visit_statements(node, new_scope)
    }

    fn visit_statement(&mut self, node: &'tree Statement) -> Result<RuntimeResult<'tree>> {
        match node {
            Statement::Var(node) => self.visit_var_stmt(node),
            Statement::Function(node) => self.visit_function_decl(node),
            Statement::Class(node) => self.visit_class_decl(node),
            Statement::Break(node) => self.visit_break_stmt(node),
            Statement::Continue(node) => self.visit_continue_stmt(node),
            Statement::Return(node) => self.visit_return_stmt(node),
            Statement::Expr(node) => self.visit_expression(node),
        }
    }

    fn visit_var_stmt(&mut self, node: &'tree VarStmt) -> Result<RuntimeResult<'tree>> {
        let val = match &node.expr {
            Some(node) => try_visit!(self.visit_expression(node)?),
            None => Value::Null.wrapped(),
        };
        self.add_var(&node.ident, val);
        Ok(RuntimeResult::new(None))
    }

    fn visit_function_decl(&mut self, node: &'tree FunctionDecl) -> Result<RuntimeResult<'tree>> {
        self.add_var(
            &node.ident,
            Value::Function {
                args: &node.args,
                block: &node.block,
            }
            .wrapped(),
        );
        Ok(RuntimeResult::new(None))
    }

    fn visit_class_decl(&mut self, node: &'tree ClassDecl) -> Result<RuntimeResult<'tree>> {
        let class = Value::Null.wrapped();
        let mut statics: HashMap<&str, _> = HashMap::new();
        let mut non_statics = vec![];
        for member in &node.block.members {
            match (member.is_static, &member.kind) {
                (true, MemberKind::Attribute(node)) => {
                    statics.insert(
                        &node.ident,
                        match &node.expr {
                            Some(node) => try_visit!(self.visit_expression(node)?),
                            None => Value::Null.wrapped(),
                        },
                    );
                }
                (true, MemberKind::Method(node)) => {
                    statics.insert(
                        &node.ident,
                        Value::Method {
                            this: Rc::clone(&class),
                            args: &node.args,
                            block: &node.block,
                        }
                        .wrapped(),
                    );
                }
                (false, member) => {
                    non_statics.push(member);
                }
            }
        }
        *class.borrow_mut() = Value::Class {
            statics,
            non_statics,
        };
        self.add_var(&node.ident, class);
        Ok(RuntimeResult::new(None))
    }

    fn visit_break_stmt(&mut self, node: &'tree BreakStmt) -> Result<RuntimeResult<'tree>> {
        let val = match &node.expr {
            Some(node) => try_visit!(self.visit_expression(node)?),
            None => Value::Null.wrapped(),
        };
        Ok(RuntimeResult::success_break(val))
    }

    fn visit_continue_stmt(&mut self, _: &'tree ContinueStmt) -> Result<RuntimeResult<'tree>> {
        Ok(RuntimeResult::success_continue())
    }

    fn visit_return_stmt(&mut self, node: &'tree ReturnStmt) -> Result<RuntimeResult<'tree>> {
        let val = match &node.expr {
            Some(node) => try_visit!(self.visit_expression(node)?),
            None => Value::Null.wrapped(),
        };
        Ok(RuntimeResult::success_return(val))
    }

    #[inline]
    fn visit_expression(&mut self, node: &'tree Expression) -> Result<RuntimeResult<'tree>> {
        self.visit_range_expr(node)
    }

    fn visit_range_expr(&mut self, node: &'tree RangeExpr) -> Result<RuntimeResult<'tree>> {
        let mut left = try_visit!(self.visit_or_expr(&node.left)?);
        if let Some((tok, right)) = &node.right {
            let right = try_visit!(self.visit_or_expr(right)?);
            let exclusive = match tok {
                TokenKind::Dots => true,
                TokenKind::DotsInclusive => false,
                _ => unreachable!(),
            };
            let range = match (&*left.borrow(), &*right.borrow()) {
                (Value::Number(start), Value::Number(end)) => {
                    if !start.fract().is_zero() || !end.fract().is_zero() {
                        error!(
                            ValueError,
                            node.start, node.end, "Range bounds have to be integers"
                        );
                    }
                    let start = start.to_i128().unwrap();
                    let end = end.to_i128().unwrap();
                    let end = end - exclusive as i128;
                    Value::Range { start, end }.wrapped()
                }
                _ => error!(
                    TypeError,
                    node.start, node.end, "Range bounds have to be of type 'number'"
                ),
            };
            left = range;
        }
        Ok(RuntimeResult::new(Some(left)))
    }

    fn visit_or_expr(&mut self, node: &'tree OrExpr) -> Result<RuntimeResult<'tree>> {
        let base = try_visit!(self.visit_and_expr(&node.base)?);
        if !node.following.is_empty() {
            if base.borrow().is_false() {
                return Ok(RuntimeResult::new(Some(Value::Bool(false).wrapped())));
            }
            for other in &node.following {
                if try_visit!(self.visit_and_expr(other)?).borrow().is_false() {
                    return Ok(RuntimeResult::new(Some(Value::Bool(false).wrapped())));
                }
            }
            return Ok(RuntimeResult::new(Some(Value::Bool(true).wrapped())));
        }
        Ok(RuntimeResult::new(Some(base)))
    }

    fn visit_and_expr(&mut self, node: &'tree AndExpr) -> Result<RuntimeResult<'tree>> {
        let base = try_visit!(self.visit_bit_or_expr(&node.base)?);
        if !node.following.is_empty() {
            if base.borrow().is_true() {
                return Ok(RuntimeResult::new(Some(Value::Bool(true).wrapped())));
            }
            for other in &node.following {
                if try_visit!(self.visit_bit_or_expr(other)?)
                    .borrow()
                    .is_true()
                {
                    return Ok(RuntimeResult::new(Some(Value::Bool(true).wrapped())));
                }
            }
            return Ok(RuntimeResult::new(Some(Value::Bool(false).wrapped())));
        }
        Ok(RuntimeResult::new(Some(base)))
    }

    simple_expr!(visit_bit_or_expr: BitOrExpr, visit_bit_xor_expr; or);
    simple_expr!(visit_bit_xor_expr: BitXorExpr, visit_bit_and_expr; xor);
    simple_expr!(visit_bit_and_expr: BitAndExpr, visit_eq_expr; and);

    fn visit_eq_expr(&mut self, node: &'tree EqExpr) -> Result<RuntimeResult<'tree>> {
        let left = try_visit!(self.visit_rel_expr(&node.left)?);
        let out = if let Some((tok, right)) = &node.right {
            let right = try_visit!(self.visit_rel_expr(right)?);
            Value::Bool(match tok {
                TokenKind::Equal => left == right,
                TokenKind::NotEqual => left != right,
                _ => unreachable!(),
            })
            .wrapped()
        } else {
            left
        };
        Ok(RuntimeResult::new(Some(out)))
    }

    fn visit_rel_expr(&mut self, node: &'tree RelExpr) -> Result<RuntimeResult<'tree>> {
        let left = try_visit!(self.visit_shift_expr(&node.left)?);
        let out = if let Some((tok, right)) = &node.right {
            let right = try_visit!(self.visit_shift_expr(right)?);
            match tok {
                TokenKind::LessThan => left.borrow().lt(&right.borrow(), &node.start, &node.end),
                TokenKind::LessThanOrEqual => {
                    left.borrow().le(&right.borrow(), &node.start, &node.end)
                }
                TokenKind::GreaterThan => left.borrow().gt(&right.borrow(), &node.start, &node.end),
                TokenKind::GreaterThanOrEqual => {
                    left.borrow().ge(&right.borrow(), &node.start, &node.end)
                }
                _ => unreachable!(),
            }?
            .wrapped()
        } else {
            left
        };
        Ok(RuntimeResult::new(Some(out)))
    }

    simple_expr!(
        visit_shift_expr: ShiftExpr, visit_add_expr;
        ShiftLeft => shl,
        ShiftRight => shr,
    );

    simple_expr!(
        visit_add_expr: AddExpr, visit_mul_expr;
        Plus => add,
        Minus => sub,
    );

    simple_expr!(
        visit_mul_expr: MulExpr, visit_unary_expr;
        Star => mul,
        Slash => div,
        Rem => rem,
        Backslash => div_floor,
    );

    fn visit_unary_expr(&mut self, node: &'tree UnaryExpr) -> Result<RuntimeResult<'tree>> {
        match node {
            UnaryExpr::Unary {
                start,
                end,
                operator,
                expr,
            } => {
                let base = try_visit!(self.visit_unary_expr(expr)?);
                let out = match operator {
                    TokenKind::Plus => {
                        Value::Number(Decimal::ZERO).add(&base.borrow(), start, end)?
                    }
                    TokenKind::Minus => {
                        Value::Number(Decimal::ZERO).sub(&base.borrow(), start, end)?
                    }
                    TokenKind::Not => Value::Bool(base.borrow().is_false()),
                    _ => unreachable!(),
                }
                .wrapped();
                Ok(RuntimeResult::new(Some(out)))
            }
            UnaryExpr::Done(node) => self.visit_exp_expr(node),
        }
    }

    fn visit_exp_expr(&mut self, node: &'tree ExpExpr) -> Result<RuntimeResult<'tree>> {
        let mut base = try_visit!(self.visit_assign_expr(&node.base)?);
        if let Some(exponent) = &node.exponent {
            let exponent = try_visit!(self.visit_unary_expr(exponent)?);
            let out = base
                .borrow()
                .pow(&exponent.borrow(), &node.start, &node.end)?
                .wrapped();
            base = out;
        }
        Ok(RuntimeResult::new(Some(base)))
    }

    fn visit_assign_expr(&mut self, node: &'tree AssignExpr) -> Result<RuntimeResult<'tree>> {
        let left = try_visit!(self.visit_call_expr(&node.left)?);
        if let Some((tok, right)) = &node.right {
            let left_type = types::type_of(&left.borrow());
            if let Type::Class | Type::Object | Type::Range = left_type {
                error!(
                    TypeError,
                    node.start, node.end, "Cannot reassign type '{}'", left_type,
                );
            }
            let right = try_visit!(self.visit_expression(right)?);
            let right_type = types::type_of(&right.borrow());
            if left_type != right_type && left_type != Type::Null && right_type != Type::Null {
                error!(
                    TypeError,
                    node.start,
                    node.end,
                    "Cannot change type by reassigning, create a new variable instead",
                );
            }
            let new_value = match tok {
                TokenKind::Assign => right.borrow().clone(),
                TokenKind::StarAssign => {
                    left.borrow().mul(&right.borrow(), &node.start, &node.end)?
                }
                TokenKind::SlashAssign => {
                    left.borrow().div(&right.borrow(), &node.start, &node.end)?
                }
                TokenKind::BackslashAssign => {
                    left.borrow()
                        .div_floor(&right.borrow(), &node.start, &node.end)?
                }
                TokenKind::RemAssign => {
                    left.borrow().rem(&right.borrow(), &node.start, &node.end)?
                }
                TokenKind::PlusAssign => {
                    left.borrow().add(&right.borrow(), &node.start, &node.end)?
                }
                TokenKind::MinusAssign => {
                    left.borrow().sub(&right.borrow(), &node.start, &node.end)?
                }
                TokenKind::ShiftLeftAssign => {
                    left.borrow().shl(&right.borrow(), &node.start, &node.end)?
                }
                TokenKind::ShiftRightAssign => {
                    left.borrow().shr(&right.borrow(), &node.start, &node.end)?
                }
                TokenKind::BitAndAssign => {
                    left.borrow().and(&right.borrow(), &node.start, &node.end)?
                }
                TokenKind::BitXorAssign => {
                    left.borrow().xor(&right.borrow(), &node.start, &node.end)?
                }
                TokenKind::BitOrAssign => {
                    left.borrow().or(&right.borrow(), &node.start, &node.end)?
                }
                _ => unreachable!(),
            };
            *left.borrow_mut() = new_value;
        }
        Ok(RuntimeResult::new(Some(left)))
    }

    fn visit_call_expr(&mut self, node: &'tree CallExpr) -> Result<RuntimeResult<'tree>> {
        let mut base = try_visit!(self.visit_member_expr(&node.base)?);
        for part in &node.following {
            let out = match part {
                CallPart::Args(args) => self.call_value(&base, args, &node.start, &node.end)?,
                CallPart::Member(MemberPart::Field(ident)) => {
                    Value::get_field(&base, ident, &node.start, &node.end)?
                }
            };
            base = out;
        }
        Ok(RuntimeResult::new(Some(base)))
    }

    pub fn call_value(
        &mut self,
        value: &WrappedValue<'tree>,
        call_args: &'tree Args,
        start: &Location,
        end: &Location,
    ) -> Result<WrappedValue<'tree>> {
        match &*value.borrow() {
            // TODO: dedup code
            Value::Function { args, block } => {
                if args.len() != call_args.len() {
                    error!(
                        TypeError,
                        *start,
                        *end,
                        "Function takes {} arguments, however {} were supllied",
                        args.len(),
                        call_args.len()
                    );
                }
                self.push_scope();
                for (idx, arg) in args.iter().enumerate() {
                    let val = self.visit_expression(&call_args[idx])?.take_value();
                    self.add_var(arg, val);
                }
                let res = self.visit_block(block, false)?;
                self.pop_scope();
                Ok(if let Some(val) = res.return_value {
                    val
                } else {
                    res.take_value()
                })
            }
            Value::Method { this, args, block } => {
                if args.len() != call_args.len() {
                    error!(
                        TypeError,
                        *start,
                        *end,
                        "Function takes {} arguments, however {} were supllied",
                        args.len(),
                        call_args.len()
                    );
                }
                self.push_scope();
                self.add_var("this", Rc::clone(this));
                for (idx, arg) in args.iter().enumerate() {
                    let val = self.visit_expression(&call_args[idx])?.take_value();
                    self.add_var(arg, val);
                }
                let res = self.visit_block(block, false)?;
                self.pop_scope();
                Ok(if let Some(val) = res.return_value {
                    val
                } else {
                    res.take_value()
                })
            }
            Value::BuiltIn(func) => {
                let mut args = vec![];
                for arg in call_args {
                    args.push(self.visit_expression(arg)?.take_value());
                }

                let out = match func {
                    BuiltIn::Function(func) => func(args, start, end),
                    BuiltIn::Method(this, func) => func(this, args, start, end),
                    BuiltIn::Print(newline) => {
                        built_in::print(args, &mut self.stdout, start, end, *newline)
                    }
                    BuiltIn::Exit => {
                        built_in::exit(args, self.exit_callback.take().unwrap(), start, end)
                    }
                }?;
                Ok(out.wrapped())
            }
            Value::Class { non_statics, .. } => {
                if !call_args.is_empty() {
                    error!(
                        TypeError,
                        *start,
                        *end,
                        "Class constructors take no arguments, however {} were supplied",
                        call_args.len(),
                    );
                }
                let object = Value::Null.wrapped();
                let mut fields: HashMap<&str, _> = HashMap::new();
                for member in non_statics {
                    match member {
                        MemberKind::Attribute(node) => {
                            fields.insert(
                                &node.ident,
                                match &node.expr {
                                    Some(node) => self.visit_expression(node)?.take_value(),
                                    None => Value::Null.wrapped(),
                                },
                            );
                        }
                        MemberKind::Method(node) => {
                            fields.insert(
                                &node.ident,
                                Value::Method {
                                    this: Rc::clone(&object),
                                    args: &node.args,
                                    block: &node.block,
                                }
                                .wrapped(),
                            );
                        }
                    }
                }
                *object.borrow_mut() = Value::Object(fields);
                Ok(object)
            }
            _ => error!(
                TypeError,
                *start,
                *end,
                "Type '{}' is not callable",
                types::type_of(&value.borrow()),
            ),
        }
    }

    fn visit_member_expr(&mut self, node: &'tree MemberExpr) -> Result<RuntimeResult<'tree>> {
        let mut base = try_visit!(self.visit_atom(&node.base)?);
        for part in &node.following {
            let out = match part {
                MemberPart::Field(ident) => Value::get_field(&base, ident, &node.start, &node.end)?,
            };
            base = out;
        }
        Ok(RuntimeResult::new(Some(base)))
    }

    fn visit_atom(&mut self, node: &'tree Atom) -> Result<RuntimeResult<'tree>> {
        let out = match node {
            Atom::Number(val) => Value::Number(*val).wrapped(),
            Atom::Bool(val) => Value::Bool(*val).wrapped(),
            Atom::String(val) => Value::String(val.clone()).wrapped(),
            Atom::Null => Value::Null.wrapped(),
            Atom::Identifier { start, end, name } => Rc::clone(self.get_var(name, (start, end))?.0),
            Atom::Expr(node) => try_visit!(self.visit_expression(node)?),
            Atom::IfExpr(node) => try_visit!(self.visit_if_expr(node)?),
            Atom::ForExpr(node) => try_visit!(self.visit_for_expr(node)?),
            Atom::WhileExpr(node) => try_visit!(self.visit_while_expr(node)?),
            Atom::LoopExpr(node) => try_visit!(self.visit_loop_expr(node)?),
            Atom::FunExpr(node) => try_visit!(self.visit_fun_expr(node)?),
            Atom::ClassExpr(node) => try_visit!(self.visit_class_expr(node)?),
            Atom::TryExpr(node) => try_visit!(self.visit_try_expr(node)?),
            Atom::BlockExpr(node) => try_visit!(self.visit_block_expr(node)?),
        };
        Ok(RuntimeResult::new(Some(out)))
    }

    fn visit_if_expr(&mut self, node: &'tree IfExpr) -> Result<RuntimeResult<'tree>> {
        let cond = try_visit!(self.visit_expression(&node.cond)?);
        let out = if cond.borrow().is_true() {
            try_visit!(self.visit_block(&node.block, true)?)
        } else if let Some(block) = &node.else_block {
            try_visit!(self.visit_block(block, true)?)
        } else {
            Value::Null.wrapped()
        };
        Ok(RuntimeResult::new(Some(out)))
    }

    fn visit_for_expr(&mut self, node: &'tree ForExpr) -> Result<RuntimeResult<'tree>> {
        let iter = try_visit!(self.visit_expression(&node.iter)?);
        let iter = iter.borrow();
        let iter = iter.to_iter(&node.start, &node.end)?;
        let mut out = Value::Null.wrapped();
        for item in iter {
            self.push_scope();
            self.add_var(&node.ident, item.wrapped());
            let res = self.visit_block(&node.block, false)?;
            if res.should_continue {
                continue;
            } else if let Some(val) = res.break_value {
                out = val;
            } else if res.return_value.is_some() {
                return Ok(res);
            }
        }
        Ok(RuntimeResult::new(Some(out)))
    }

    fn visit_while_expr(&mut self, node: &'tree WhileExpr) -> Result<RuntimeResult<'tree>> {
        let out = loop {
            let cond = try_visit!(self.visit_expression(&node.cond)?);
            if cond.borrow().is_false() {
                break Value::Null.wrapped();
            }

            let res = self.visit_block(&node.block, true)?;
            if res.should_continue {
                continue;
            } else if let Some(val) = res.break_value {
                break val;
            } else if res.return_value.is_some() {
                return Ok(res);
            }
        };
        Ok(RuntimeResult::new(Some(out)))
    }

    fn visit_loop_expr(&mut self, node: &'tree LoopExpr) -> Result<RuntimeResult<'tree>> {
        let out = loop {
            let res = self.visit_block(&node.block, true)?;
            if res.should_continue {
                continue;
            } else if let Some(val) = res.break_value {
                break val;
            } else if res.return_value.is_some() {
                return Ok(res);
            }
        };
        Ok(RuntimeResult::new(Some(out)))
    }

    fn visit_fun_expr(&mut self, node: &'tree FunExpr) -> Result<RuntimeResult<'tree>> {
        let out = Value::Function {
            args: &node.args,
            block: &node.block,
        }
        .wrapped();
        Ok(RuntimeResult::new(Some(out)))
    }

    // TODO: dedup code
    fn visit_class_expr(&mut self, node: &'tree ClassExpr) -> Result<RuntimeResult<'tree>> {
        let class = Value::Null.wrapped();
        let mut statics: HashMap<&str, _> = HashMap::new();
        let mut non_statics = vec![];
        for member in &node.block.members {
            match (member.is_static, &member.kind) {
                (true, MemberKind::Attribute(node)) => {
                    statics.insert(
                        &node.ident,
                        match &node.expr {
                            Some(node) => try_visit!(self.visit_expression(node)?),
                            None => Value::Null.wrapped(),
                        },
                    );
                }
                (true, MemberKind::Method(node)) => {
                    statics.insert(
                        &node.ident,
                        Value::Method {
                            this: Rc::clone(&class),
                            args: &node.args,
                            block: &node.block,
                        }
                        .wrapped(),
                    );
                }
                (false, member) => {
                    non_statics.push(member);
                }
            }
        }
        *class.borrow_mut() = Value::Class {
            statics,
            non_statics,
        };
        Ok(RuntimeResult::new(Some(class)))
    }

    fn visit_try_expr(&mut self, node: &'tree TryExpr) -> Result<RuntimeResult<'tree>> {
        let res = self.visit_block(&node.try_block, true);
        if let Err(e) = res {
            self.push_scope();
            self.add_var(
                &node.ident,
                // TODO: special error type
                Value::String(format!(
                    "{kind:?} at {line}:{column}  {msg}",
                    kind = e.kind,
                    line = e.start.line,
                    column = e.start.column,
                    msg = e.message,
                ))
                .wrapped(),
            );
            let out = try_visit!(self.visit_block(&node.catch_block, false)?);
            self.pop_scope();
            Ok(RuntimeResult::new(Some(out)))
        } else {
            res
        }
    }

    #[inline]
    fn visit_block_expr(&mut self, node: &'tree BlockExpr) -> Result<RuntimeResult<'tree>> {
        self.visit_block(node, true)
    }
}
