use std::{rc::Rc, str::FromStr};

use crate::{
    error::{Result, Span},
    interpreter::value::{indexing::ToListIndex, BuiltIn, Value, WrappedValue},
};
use once_cell::unsync::Lazy;
use rust_decimal::{prelude::ToPrimitive, Decimal};

macro_rules! parse_err {
    ($this:ident, $span:expr, $to:expr) => {
        error!(
            ValueError,
            $span,
            "Could not parse String '{}' to {}",
            $this.borrow(),
            $to,
        )
    };
}

macro_rules! unwrap_variant {
    ($borrow:expr, $variant:ident $($mut:tt)?) => {
        match &$($mut)? *$borrow {
            Value::$variant(val) => val,
            _ => unreachable!(),
        }
    };
}

pub struct BuiltInMethods<'tree> {
    pub(super) to_string: Lazy<WrappedValue<'tree>>,
    pub(super) to_bool: Lazy<WrappedValue<'tree>>,
    pub(super) clone: Lazy<WrappedValue<'tree>>,
    pub(super) str_to_int: Lazy<WrappedValue<'tree>>,
    pub(super) str_to_number: Lazy<WrappedValue<'tree>>,
    pub(super) str_to_bool: Lazy<WrappedValue<'tree>>,
    pub(super) str_to_bool_strict: Lazy<WrappedValue<'tree>>,
    pub(super) str_to_range: Lazy<WrappedValue<'tree>>,
    pub(super) str_to_uppercase: Lazy<WrappedValue<'tree>>,
    pub(super) str_to_lowercase: Lazy<WrappedValue<'tree>>,
    pub(super) num_to_int: Lazy<WrappedValue<'tree>>,
    pub(super) num_floor: Lazy<WrappedValue<'tree>>,
    pub(super) num_ceil: Lazy<WrappedValue<'tree>>,
    pub(super) num_round: Lazy<WrappedValue<'tree>>,
    pub(super) list_push: Lazy<WrappedValue<'tree>>,
    pub(super) list_pop: Lazy<WrappedValue<'tree>>,
    pub(super) list_insert: Lazy<WrappedValue<'tree>>,
    pub(super) list_remove: Lazy<WrappedValue<'tree>>,
}

impl<'tree> BuiltInMethods<'tree> {
    pub(crate) fn new() -> Self {
        Self {
            to_string: Lazy::new(|| Value::BuiltIn(BuiltIn::Method(to_string)).wrapped()),
            to_bool: Lazy::new(|| Value::BuiltIn(BuiltIn::Method(to_bool)).wrapped()),
            clone: Lazy::new(|| Value::BuiltIn(BuiltIn::Method(clone)).wrapped()),
            str_to_int: Lazy::new(|| Value::BuiltIn(BuiltIn::Method(str_to_int)).wrapped()),
            str_to_number: Lazy::new(|| Value::BuiltIn(BuiltIn::Method(str_to_number)).wrapped()),
            str_to_bool: Lazy::new(|| Value::BuiltIn(BuiltIn::Method(str_to_bool)).wrapped()),
            str_to_bool_strict: Lazy::new(|| {
                Value::BuiltIn(BuiltIn::Method(str_to_bool_strict)).wrapped()
            }),
            str_to_range: Lazy::new(|| Value::BuiltIn(BuiltIn::Method(str_to_range)).wrapped()),
            str_to_uppercase: Lazy::new(|| {
                Value::BuiltIn(BuiltIn::Method(str_to_uppercase)).wrapped()
            }),
            str_to_lowercase: Lazy::new(|| {
                Value::BuiltIn(BuiltIn::Method(str_to_lowercase)).wrapped()
            }),
            num_to_int: Lazy::new(|| Value::BuiltIn(BuiltIn::Method(num_to_int)).wrapped()),
            num_floor: Lazy::new(|| Value::BuiltIn(BuiltIn::Method(num_floor)).wrapped()),
            num_ceil: Lazy::new(|| Value::BuiltIn(BuiltIn::Method(num_ceil)).wrapped()),
            num_round: Lazy::new(|| Value::BuiltIn(BuiltIn::Method(num_round)).wrapped()),
            list_push: Lazy::new(|| Value::BuiltIn(BuiltIn::Method(list_push)).wrapped()),
            list_pop: Lazy::new(|| Value::BuiltIn(BuiltIn::Method(list_pop)).wrapped()),
            list_insert: Lazy::new(|| Value::BuiltIn(BuiltIn::Method(list_insert)).wrapped()),
            list_remove: Lazy::new(|| Value::BuiltIn(BuiltIn::Method(list_remove)).wrapped()),
        }
    }
}

fn to_string<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: &Span,
) -> Result<WrappedValue<'tree>> {
    expect_len!(args, 0, "toString", span);
    Ok(Value::String(this.borrow().to_string()).wrapped())
}

fn to_bool<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: &Span,
) -> Result<WrappedValue<'tree>> {
    expect_len!(args, 0, "toBool", span);
    Ok(Value::Bool(this.borrow().is_true()).wrapped())
}

fn clone<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: &Span,
) -> Result<WrappedValue<'tree>> {
    expect_len!(args, 0, "clone", span);
    Ok(this.borrow().clone().wrapped())
}

fn str_to_int<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: &Span,
) -> Result<WrappedValue<'tree>> {
    let borrow = this.borrow();
    let str = unwrap_variant!(borrow, String);
    if args.len() > 1 {
        expect_len!(args, 1, "toInt", span);
    }
    let radix = match args.get(0) {
        Some(val) => match &*val.borrow() {
            Value::Number(radix) => match radix.to_u32() {
                Some(num) => num,
                None => error!(ValueError, *span, "Invalid radix: {}", radix),
            },
            _ => error!(TypeError, *span, "The radix has to be of type 'number'",),
        },
        None => 10,
    };
    if !(2..=36).contains(&radix) {
        error!(
            ValueError,
            *span, "Radix has to be in 2..=36, got {}", radix,
        );
    }
    match Decimal::from_str_radix(str, radix) {
        Ok(num) if num.fract().is_zero() => Ok(Value::Number(num).wrapped()),
        _ => parse_err!(this, *span, "integer"),
    }
}

fn str_to_number<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: &Span,
) -> Result<WrappedValue<'tree>> {
    let borrow = this.borrow();
    let str = unwrap_variant!(borrow, String);
    expect_len!(args, 0, "toNumber", span);
    match Decimal::from_str(str) {
        Ok(num) => Ok(Value::Number(num).wrapped()),
        Err(_) => parse_err!(this, *span, "number"),
    }
}

fn str_to_bool<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: &Span,
) -> Result<WrappedValue<'tree>> {
    let borrow = this.borrow();
    let str = unwrap_variant!(borrow, String);
    expect_len!(args, 0, "toBool", span);
    Ok(Value::Bool(str.to_ascii_lowercase() == "true").wrapped())
}

fn str_to_bool_strict<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: &Span,
) -> Result<WrappedValue<'tree>> {
    let borrow = this.borrow();
    let str = unwrap_variant!(borrow, String);
    expect_len!(args, 0, "toBoolStrict", span);
    Ok(match str.as_str() {
        "true" => Value::Bool(true).wrapped(),
        "false" => Value::Bool(false).wrapped(),
        _ => parse_err!(this, *span, "bool"),
    })
}

fn str_to_range<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: &Span,
) -> Result<WrappedValue<'tree>> {
    let borrow = this.borrow();
    let str = unwrap_variant!(borrow, String);
    expect_len!(args, 0, "toRange", span);
    let mut split = str.split("..");
    let left = split.next().unwrap();
    let mut right = match split.next() {
        Some(right) => right,
        None => parse_err!(this, *span, "range"),
    };
    let inclusive = right.starts_with('=');
    if inclusive {
        right = &right[1..];
    }

    let (left, right) = match (i128::from_str(left), i128::from_str(right)) {
        (Ok(left), Ok(right)) => (left, right),
        _ => parse_err!(this, *span, "range"),
    };
    Ok(Value::Range {
        start: left,
        end: right - !inclusive as i128,
    }
    .wrapped())
}

fn str_to_uppercase<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: &Span,
) -> Result<WrappedValue<'tree>> {
    let borrow = this.borrow();
    let str = unwrap_variant!(borrow, String);
    expect_len!(args, 0, "toUppercase", span);
    Ok(Value::String(str.to_ascii_uppercase()).wrapped())
}

fn str_to_lowercase<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: &Span,
) -> Result<WrappedValue<'tree>> {
    let borrow = this.borrow();
    let str = unwrap_variant!(borrow, String);
    expect_len!(args, 0, "toLowercase", span);
    Ok(Value::String(str.to_ascii_lowercase()).wrapped())
}

fn num_to_int<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: &Span,
) -> Result<WrappedValue<'tree>> {
    let borrow = this.borrow();
    let num = unwrap_variant!(borrow, Number);
    expect_len!(args, 0, "toInt", span);
    Ok(Value::Number(num.trunc()).wrapped())
}

fn num_floor<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: &Span,
) -> Result<WrappedValue<'tree>> {
    let borrow = this.borrow();
    let num = unwrap_variant!(borrow, Number);
    expect_len!(args, 0, "floor", span);
    Ok(Value::Number(num.floor()).wrapped())
}

fn num_ceil<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: &Span,
) -> Result<WrappedValue<'tree>> {
    let borrow = this.borrow();
    let num = unwrap_variant!(borrow, Number);
    expect_len!(args, 0, "ceil", span);
    Ok(Value::Number(num.ceil()).wrapped())
}

fn num_round<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: &Span,
) -> Result<WrappedValue<'tree>> {
    let borrow = this.borrow();
    let num = unwrap_variant!(borrow, Number);
    expect_len!(args, 0, "round", span);
    Ok(Value::Number(num.round()).wrapped())
}

fn list_push<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: &Span,
) -> Result<WrappedValue<'tree>> {
    let mut borrow = this.borrow_mut();
    let list = unwrap_variant!(borrow, List mut);
    expect_len!(args, 1, "push", span);
    list.push(Rc::clone(&args[0]));
    Ok(Value::Null.wrapped())
}

fn list_pop<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: &Span,
) -> Result<WrappedValue<'tree>> {
    let mut borrow = this.borrow_mut();
    let list = unwrap_variant!(borrow, List mut);
    expect_len!(args, 0, "pop", span);
    Ok(list.pop().unwrap_or_else(|| Value::Null.wrapped()))
}

fn list_insert<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: &Span,
) -> Result<WrappedValue<'tree>> {
    let mut borrow = this.borrow_mut();
    let list = unwrap_variant!(borrow, List mut);
    expect_len!(args, 2, "insert", span);
    let index = args[0].borrow().to_list_index(list.len(), span)?;
    list.insert(index, Rc::clone(&args[1]));
    Ok(Value::Null.wrapped())
}

fn list_remove<'tree>(
    this: &WrappedValue<'tree>,
    args: Vec<WrappedValue<'tree>>,
    span: &Span,
) -> Result<WrappedValue<'tree>> {
    let mut borrow = this.borrow_mut();
    let list = unwrap_variant!(borrow, List mut);
    expect_len!(args, 1, "remove", span);
    list.remove(args[0].borrow().to_list_index(list.len(), span)?);
    Ok(Value::Null.wrapped())
}
