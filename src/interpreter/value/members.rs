mod built_in;

use rust_decimal::Decimal;
use super::{Value, types::type_of, BuiltIn};
use crate::error::{Result, Location};

pub trait Members {
    fn get_member(&self, name: &String, start_loc: Location, end_loc: Location) -> Result<Value>;
    fn get_shared_member(&self, name: &String, start_loc: Location, end_loc: Location) -> Result<Value>;
    fn set_member(&self, name: &String, new_val: Value, start_loc: Location, end_loc: Location) -> Result<Value>;
}

impl Members for Value {
    fn get_member(&self, name: &String, start_loc: Location, end_loc: Location) -> Result<Value> {
        return Ok(match self {
            Value::Object(members) => match members.get(name) {
                Some(value) => value.clone(),
                None => self.get_shared_member(name, start_loc, end_loc)?,
            },
            Value::Class(members) => match members.get(name) {
                Some(value) => match value {
                    Value::Function(true, _, _) => value.clone(),
                    _ => self.get_shared_member(name, start_loc, end_loc)?,
                },
                None => self.get_shared_member(name, start_loc, end_loc)?,
            },
            Value::String(val) => match name.as_str() {
                "length"       => Value::Number(Decimal::from(val.len())),
                "toInt"        => Value::BuiltIn(BuiltIn::Method(built_in::str_to_int)),
                "toDecimal"    => Value::BuiltIn(BuiltIn::Method(built_in::str_to_decimal)),
                "toBool"       => Value::BuiltIn(BuiltIn::Method(built_in::str_to_bool)),
                "toBoolStrict" => Value::BuiltIn(BuiltIn::Method(built_in::str_to_bool_strict)),
                "toRange"      => Value::BuiltIn(BuiltIn::Method(built_in::str_to_range)),
                "toUppercase"  => Value::BuiltIn(BuiltIn::Method(built_in::str_to_uppercase)),
                "toLowercase"  => Value::BuiltIn(BuiltIn::Method(built_in::str_to_lowercase)),
                _ => self.get_shared_member(name, start_loc, end_loc)?,
            },
            Value::Number(_) => match name.as_str() {
                "toInt" => Value::BuiltIn(BuiltIn::Method(built_in::num_to_int)),
                "floor" => Value::BuiltIn(BuiltIn::Method(built_in::num_floor)),
                "ceil"  => Value::BuiltIn(BuiltIn::Method(built_in::num_ceil)),
                "round" => Value::BuiltIn(BuiltIn::Method(built_in::num_round)),
                _ => self.get_shared_member(name, start_loc, end_loc)?,
            },
            _ => self.get_shared_member(name, start_loc, end_loc)?,
        });
    }

    fn get_shared_member(&self, name: &String, start_loc: Location, end_loc: Location) -> Result<Value> {
        return Ok(match name.as_str() {
            "toString" => Value::BuiltIn(BuiltIn::Method(built_in::to_string)),
            "toBool" => Value::BuiltIn(BuiltIn::Method(built_in::to_bool)),
            _ => error!(ReferenceError, start_loc, end_loc, "Type {} has no member called {}", type_of(self), name),
        });
    }

    fn set_member(&self, name: &String, new_val: Value, start_loc: Location, end_loc: Location) -> Result<Value> {
        return Ok(match self {
            Value::Object(members) => if members.contains_key(name) {
                let mut new_members = members.clone();
                new_members.insert(name.clone(), new_val);
                Value::Object(new_members)
            } else { error!(ReferenceError, start_loc, end_loc, "Object has no member called {}", name); },
            _ => error!(TypeError, start_loc, end_loc, "Cannot assign to member {}", name),
        })
    }
}
