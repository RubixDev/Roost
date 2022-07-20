mod built_in;

use std::rc::Rc;

use crate::error::{Location, Result};

use super::{types, BuiltIn, Value, WrappedValue};

impl<'tree> Value<'tree> {
    pub fn get_field(
        this: &WrappedValue<'tree>,
        name: &str,
        start: &Location,
        end: &Location,
    ) -> Result<WrappedValue<'tree>> {
        Ok(match &*this.borrow() {
            Value::Object(fields)
            | Value::Class {
                statics: fields, ..
            } => match fields.get(name) {
                Some(field) => Rc::clone(field),
                None => Self::get_common_field(this, name, start, end)?,
            },
            Value::String(val) => match name {
                "length" => Value::Number(val.len().into()).wrapped(),
                "toInt" => {
                    Value::BuiltIn(BuiltIn::Method(Rc::clone(this), built_in::str_to_int)).wrapped()
                }
                "toNumber" => {
                    Value::BuiltIn(BuiltIn::Method(Rc::clone(this), built_in::str_to_number))
                        .wrapped()
                }
                "toBool" => Value::BuiltIn(BuiltIn::Method(Rc::clone(this), built_in::str_to_bool))
                    .wrapped(),
                "toBoolStrict" => Value::BuiltIn(BuiltIn::Method(
                    Rc::clone(this),
                    built_in::str_to_bool_strict,
                ))
                .wrapped(),
                "toRange" => {
                    Value::BuiltIn(BuiltIn::Method(Rc::clone(this), built_in::str_to_range))
                        .wrapped()
                }
                "toUppercase" => {
                    Value::BuiltIn(BuiltIn::Method(Rc::clone(this), built_in::str_to_uppercase))
                        .wrapped()
                }
                "toLowercase" => {
                    Value::BuiltIn(BuiltIn::Method(Rc::clone(this), built_in::str_to_lowercase))
                        .wrapped()
                }
                _ => Self::get_common_field(this, name, start, end)?,
            },
            Value::Number(_) => match name {
                "toInt" => {
                    Value::BuiltIn(BuiltIn::Method(Rc::clone(this), built_in::num_to_int)).wrapped()
                }
                "floor" => {
                    Value::BuiltIn(BuiltIn::Method(Rc::clone(this), built_in::num_floor)).wrapped()
                }
                "ceil" => {
                    Value::BuiltIn(BuiltIn::Method(Rc::clone(this), built_in::num_ceil)).wrapped()
                }
                "round" => {
                    Value::BuiltIn(BuiltIn::Method(Rc::clone(this), built_in::num_round)).wrapped()
                }
                _ => Self::get_common_field(this, name, start, end)?,
            },
            _ => Self::get_common_field(this, name, start, end)?,
        })
    }

    fn get_common_field(
        this: &WrappedValue<'tree>,
        name: &str,
        start: &Location,
        end: &Location,
    ) -> Result<WrappedValue<'tree>> {
        Ok(match name {
            "toString" => {
                Value::BuiltIn(BuiltIn::Method(Rc::clone(this), built_in::to_string)).wrapped()
            }
            "toBool" => {
                Value::BuiltIn(BuiltIn::Method(Rc::clone(this), built_in::to_bool)).wrapped()
            }
            "clone" => Value::BuiltIn(BuiltIn::Method(Rc::clone(this), built_in::clone)).wrapped(),
            _ => error!(
                ReferenceError,
                *start,
                *end,
                "Type '{}' has no member called '{}'",
                types::type_of(&this.borrow()),
                name,
            ),
        })
    }
}
