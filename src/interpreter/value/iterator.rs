use rust_decimal::{prelude::{ToPrimitive, FromPrimitive}, Decimal};
use super::{Value, types::type_of};
use crate::error::{Result, Location};

pub trait ToIterator where Self: Sized {
    fn to_iter(&self, location: Location) -> Result<Iterator>;
}

impl ToIterator for Value {
    fn to_iter(&self, location: Location) -> Result<Iterator> {
        match self {
            Value::String(value) => { return Ok(Iterator::from(value)); },
            Value::Range(start, end) => {
                let range = start.min(end).to_i128().unwrap()..=start.max(end).to_i128().unwrap();
                let mut vec: Vec<_> = range.map(|it| Value::Number(Decimal::from_i128(it).unwrap())).collect();
                if start > end { vec.reverse(); }
                return Ok(Iterator::new(vec));
            },
            _ => error!(TypeError, location, "Cannot iterate over type {}", type_of(self)),
        }
    }
}

pub struct Iterator {
    items: Vec<Value>,
    index: usize,
}

impl Iterator {
    pub fn new(items: Vec<Value>) -> Iterator {
        Iterator { items, index: 0 }
    }

    pub fn next(&mut self) -> Option<&Value> {
        let out = self.items.get(self.index);
        self.index += 1;
        return out;
    }
}

impl From<&String> for Iterator {
    fn from(str: &String) -> Self {
        Iterator::new(str.chars().map(|it| Value::String(it.to_string())).collect())
    }
}
