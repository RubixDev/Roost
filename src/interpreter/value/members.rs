mod built_in;

use rust_decimal::Decimal;
use super::{Value, types::type_of, BuiltIn};
use crate::error::{Result, Location};

macro_rules! not_found {
    ($self:ident, $name:ident, $start:ident, $end:ident) => {
        error!(ReferenceError, $start, $end, "Type {} has no member called {}", type_of($self), $name)
    };
}

pub trait Members {
    fn get_member(&self, name: &String, start_loc: Location, end_loc: Location) -> Result<Value>;
    fn get_shared_member(&self, name: &String, start_loc: Location, end_loc: Location) -> Result<Value>;
}

impl Members for Value {
    fn get_member(&self, name: &String, start_loc: Location, end_loc: Location) -> Result<Value> {
        return Ok(match self {
            Value::String(val) => match name.as_str() {
                "length" => Value::Number(Decimal::from(val.len())),
                "toInt" => Value::BuiltIn(BuiltIn::Method(built_in::to_int)),
                "toDecimal" => Value::BuiltIn(BuiltIn::Method(built_in::to_decimal)),
                "toBool" => Value::BuiltIn(BuiltIn::Method(built_in::str_to_bool)),
                "toBoolStrict" => Value::BuiltIn(BuiltIn::Method(built_in::to_bool_strict)),
                "toRange" => Value::BuiltIn(BuiltIn::Method(built_in::to_range)),
                "toUppercase" => Value::BuiltIn(BuiltIn::Method(built_in::to_uppercase)),
                "toLowercase" => Value::BuiltIn(BuiltIn::Method(built_in::to_lowercase)),
                _ => self.get_shared_member(name, start_loc, end_loc)?,
            },
            _ => self.get_shared_member(name, start_loc, end_loc)?,
        });
    }

    fn get_shared_member(&self, name: &String, start_loc: Location, end_loc: Location) -> Result<Value> {
        return Ok(match name.as_str() {
            "toString" => Value::BuiltIn(BuiltIn::Method(built_in::to_string)),
            "toBool" => Value::BuiltIn(BuiltIn::Method(built_in::to_bool)),
            _ => not_found!(self, name, start_loc, end_loc),
        });
    }
}
