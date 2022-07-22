pub mod bitwise_operations;
pub mod iterator;
pub mod mathematical_operations;
pub mod members;
pub mod relational_operations;
pub mod truth;
pub mod types;

use crate::{
    error::{Result, Span},
    nodes::{Block, MemberKind},
};
use rust_decimal::Decimal;
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
    rc::Rc,
};

pub type WrappedValue<'tree> = Rc<RefCell<Value<'tree>>>;

#[derive(PartialEq, Clone)]
pub enum Value<'tree> {
    Number(Decimal),
    Bool(bool),
    String(String),
    Range {
        start: i128,
        end: i128,
    },
    Function {
        args: &'tree [String],
        block: &'tree Block,
    },
    Method {
        this: WrappedValue<'tree>,
        args: &'tree [String],
        block: &'tree Block,
    },
    BuiltIn(BuiltIn<'tree>),
    Class {
        statics: HashMap<&'tree str, WrappedValue<'tree>>,
        non_statics: Vec<&'tree MemberKind>,
    },
    Object(HashMap<&'tree str, WrappedValue<'tree>>),
    Null,
}

impl<'tree> Value<'tree> {
    pub fn wrapped(self) -> WrappedValue<'tree> {
        Rc::new(RefCell::new(self))
    }
}

#[derive(Clone)]
pub enum BuiltIn<'tree> {
    Function(fn(args: Vec<WrappedValue<'tree>>, span: Span) -> Result<Value<'tree>>),
    Method(
        WrappedValue<'tree>,
        fn(
            this: &WrappedValue<'tree>,
            args: Vec<WrappedValue<'tree>>,
            span: Span,
        ) -> Result<Value<'tree>>,
    ),
    Print {
        newline: bool,
        stderr: bool,
    },
    Exit,
    Debug,
}

impl PartialEq for BuiltIn<'_> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

macro_rules! dbg_map {
    ($map:ident) => {
        dbg_map!(@inner $map, "", "", "")
    };
    (:? $map:ident) => {
        dbg_map!(@inner $map, "\x1b[31m", "\x1b[0m", ":?")
    };
    (@inner $map:ident, $col:literal, $reset:literal, $dbg:literal) => {
        $map.iter()
            .map(|(k, v)| {
                format!(
                    concat!("    ", $col, "{k}", $reset, " = {v},\n"),
                    k = k,
                    v = format!(concat!("{", $dbg, "}"), v.borrow())
                        .split('\n')
                        .collect::<Vec<_>>()
                        .join("\n    "),
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(value) => Display::fmt(&value, f),
            Value::Bool(value) => Display::fmt(&value, f),
            Value::String(value) => Display::fmt(&value, f),
            Value::Range { start, end } => write!(f, "{start}..={end}"),
            Value::Function { .. } | Value::Method { .. } | Value::BuiltIn(..) => {
                write!(f, "<function>")
            }
            Value::Class { statics, .. } => write!(f, "<class> {{\n{}}}", dbg_map!(statics)),
            Value::Object(fields) => write!(f, "<object> {{\n{}}}", dbg_map!(fields)),
            Value::Null => write!(f, "null"),
        }
    }
}

impl Debug for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(value) => write!(f, "\x1b[33m{value}\x1b[0m"),
            Value::Bool(value) => write!(f, "\x1b[34m{value}\x1b[0m"),
            Value::String(value) => write!(f, "\x1b[32m'{value}'\x1b[0m"),
            Value::Range { start, end } => {
                write!(f, "\x1b[33m{start}\x1b[0m..=\x1b[33m{end}\x1b[0m")
            }
            Value::Function { .. } | Value::Method { .. } | Value::BuiltIn(..) => {
                write!(f, "\x1b[1m<function>\x1b[0m")
            }
            Value::Class { statics, .. } => {
                write!(f, "\x1b[1m<class>\x1b[0m {{\n{}}}", dbg_map!(:? statics))
            }
            Value::Object(fields) => {
                write!(f, "\x1b[1m<object>\x1b[0m {{\n{}}}", dbg_map!(:? fields))
            }
            Value::Null => write!(f, "\x1b[90mnull\x1b[0m"),
        }
    }
}

pub trait ToValue {
    fn to_value<'tree>(&self) -> Value<'tree>;
}
