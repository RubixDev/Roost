use super::{Value, Range, types::type_of};

pub trait ToIterator where Self: Sized {
    fn to_iter(&self) -> Iterator;
}

impl ToIterator for Value {
    fn to_iter(&self) -> Iterator {
        match self {
            Value::String(value) => { return Iterator::from(value); },
            Value::Range(value) => { return Iterator::from(value.clone()); },
            _ => panic!("TypeError at position {{}}: Cannot iterate over type {}", type_of(self)),
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

impl From<Range> for Iterator {
    fn from(range: Range) -> Self {
        match range {
            Range::Int(inclusive, start, end) => {
                let end = if inclusive || start == end { end } else {
                    if start >= end { end + 1 } else { end - 1 }
                };
                let range = start.min(end)..=start.max(end);
                let mut vec: Vec<_> = range.map(|it| Value::Int(it)).collect();
                if end < start { vec.reverse(); }
                return Iterator::new(vec);
            },
            Range::Long(inclusive, start, end) => {
                let end = if inclusive || start == end { end } else {
                    if start >= end { end + 1 } else { end - 1 }
                };
                let mut vec = vec![];
                let mut i = start.clone().min(end.clone());
                let max = start.clone().max(end.clone());
                while &i <= &max {
                    vec.push(Value::Long(i.clone()));
                    i += 1;
                }
                if end < start { vec.reverse(); }
                return Iterator::new(vec);
            },
        }
    }
}
