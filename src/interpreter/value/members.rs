mod built_in;
pub(crate) use built_in::BuiltInMethods;

use std::{borrow::Cow, rc::Rc};

use crate::error::{Result, Span};

use super::{types, Value, WrappedValue};

impl<'tree> Value<'tree> {
    pub fn get_field(
        this: &WrappedValue<'tree>,
        name: &str,
        built_in_methods: &BuiltInMethods<'tree>,
        span: Span,
    ) -> Result<WrappedValue<'tree>> {
        Ok(match &*this.borrow() {
            Value::Object(fields)
            | Value::Class {
                statics: fields, ..
            } => match fields.get(name) {
                Some(field) => Rc::clone(field),
                None => Self::get_common_field(this, name, built_in_methods, span)?,
            },
            Value::String(val) => match name {
                "length" => Value::Number(val.len().into()).wrapped(),
                "toInt" => Rc::clone(&*built_in_methods.str_to_int),
                "toNumber" => Rc::clone(&*built_in_methods.str_to_number),
                "toBool" => Rc::clone(&*built_in_methods.str_to_bool),
                "toBoolStrict" => Rc::clone(&*built_in_methods.str_to_bool_strict),
                "toRange" => Rc::clone(&*built_in_methods.str_to_range),
                "toUppercase" => Rc::clone(&*built_in_methods.str_to_uppercase),
                "toLowercase" => Rc::clone(&*built_in_methods.str_to_lowercase),
                _ => Self::get_common_field(this, name, built_in_methods, span)?,
            },
            Value::Number(_) => match name {
                "toInt" => Rc::clone(&*built_in_methods.num_to_int),
                "floor" => Rc::clone(&*built_in_methods.num_floor),
                "ceil" => Rc::clone(&*built_in_methods.num_ceil),
                "round" => Rc::clone(&*built_in_methods.num_round),
                _ => Self::get_common_field(this, name, built_in_methods, span)?,
            },
            _ => Self::get_common_field(this, name, built_in_methods, span)?,
        })
    }

    fn get_common_field(
        this: &WrappedValue<'tree>,
        name: &str,
        built_in_methods: &BuiltInMethods<'tree>,
        span: Span,
    ) -> Result<WrappedValue<'tree>> {
        Ok(match name {
            "toString" => Rc::clone(&*built_in_methods.to_string),
            "toBool" => Rc::clone(&*built_in_methods.to_bool),
            "clone" => Rc::clone(&*built_in_methods.clone),
            _ => error!(
                ReferenceError,
                span,
                "{} has no member called '{}'",
                match &*this.borrow() {
                    Value::Class { .. } => Cow::Borrowed("Class"),
                    Value::Object(..) => Cow::Borrowed("Object"),
                    _ => Cow::Owned(format!("Type '{}'", types::type_of(&this.borrow()))),
                },
                name,
            ),
        })
    }
}
