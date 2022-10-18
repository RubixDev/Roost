pub mod bitwise_operations;
pub mod indexing;
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
        start: Option<i128>,
        end: Option<i128>,
    },
    List(Vec<WrappedValue<'tree>>),
    Function {
        args: &'tree [String],
        block: &'tree Block,
    },
    BuiltIn(BuiltIn),
    Class {
        statics: HashMap<&'tree str, WrappedValue<'tree>>,
        non_statics: Vec<&'tree MemberKind>,
    },
    Object(HashMap<&'tree str, WrappedValue<'tree>>),
    Null,
}

macro_rules! unwrap_fns {
    ($($name:ident: $([$mut:tt])? $variant:ident => $type:tt,)*) => {$(
        pub fn $name(&$($mut)? self) -> unwrap_fns!(@type $type $($mut)?) {
            unwrap_fns!(@match self, $variant, $type)
        }
    )*};
    (@type ($type:ty) $($mut:tt)?) => { &$($mut)? $type };
    (@type { $($field:ident : $type:ty),* $(,)? }) => { ($(&$type),*) };
    (@type { $($field:ident : $type:ty),* $(,)? } mut) => { ($(&mut $type),*) };
    (@match $self:ident, $variant:ident, ($type:ty)) => {
        match $self {
            Self::$variant(val) => val,
            v => unwrap_fns!(@panic v),
        }
    };
    (@match $self:ident, $variant:ident, { $($field:ident : $type:ty),* $(,)? }) => {
        match $self {
            Self::$variant { $($field),* } => ($($field),*),
            v => unwrap_fns!(@panic v),
        }
    };
    (@panic $v:ident) => { panic!("unexpected value variant `{:?}`", $v) }
}

impl<'tree> Value<'tree> {
    pub fn wrapped(self) -> WrappedValue<'tree> {
        Rc::new(RefCell::new(self))
    }

    unwrap_fns! {
        unwrap_number: Number => (Decimal),
        unwrap_string: String => (String),
        unwrap_range: Range => { start: Option<i128>, end: Option<i128> },
        unwrap_list_mut: [mut] List => (Vec<WrappedValue<'tree>>),
    }
}

#[derive(Clone)]
pub enum BuiltIn {
    Function(
        for<'tree> fn(args: Vec<WrappedValue<'tree>>, span: &Span) -> Result<WrappedValue<'tree>>,
    ),
    Method(
        for<'tree> fn(
            this: &WrappedValue<'tree>,
            args: Vec<WrappedValue<'tree>>,
            span: &Span,
        ) -> Result<WrappedValue<'tree>>,
    ),
    Print {
        newline: bool,
        stderr: bool,
    },
    Exit,
    Debug,
}

impl PartialEq for BuiltIn {
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
            Value::Range { start, end } => match (start, end) {
                (Some(start), Some(end)) => write!(f, "{start}..={end}"),
                (Some(start), None) => write!(f, "{start}.."),
                (None, Some(end)) => write!(f, "..={end}"),
                (None, None) => write!(f, ".."),
            },
            Value::List(list) => write!(
                f,
                "[{}]",
                list.iter()
                    .map(|val| val.borrow().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Function { .. } | Value::BuiltIn(..) => {
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
            Value::Range { start, end } => match (start, end) {
                (Some(start), Some(end)) => {
                    write!(f, "\x1b[33m{start}\x1b[0m..=\x1b[33m{end}\x1b[0m")
                }
                (Some(start), None) => write!(f, "\x1b[33m{start}\x1b[0m.."),
                (None, Some(end)) => write!(f, "..=\x1b[33m{end}\x1b[0m"),
                (None, None) => write!(f, ".."),
            },
            Value::List(list) => write!(
                f,
                "[{}]",
                list.iter()
                    .map(|val| format!("{:?}", val.borrow()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Function { .. } | Value::BuiltIn(..) => {
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
