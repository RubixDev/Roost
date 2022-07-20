use super::value::{Value, WrappedValue};

#[derive(Debug, Default)]
pub struct RuntimeResult<'tree> {
    pub should_continue: bool,
    pub break_value: Option<WrappedValue<'tree>>,
    pub return_value: Option<WrappedValue<'tree>>,
    pub value: Option<WrappedValue<'tree>>,
}

impl<'tree> RuntimeResult<'tree> {
    pub fn new(value: Option<WrappedValue<'tree>>) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }

    pub fn success_continue() -> Self {
        Self {
            should_continue: true,
            ..Default::default()
        }
    }

    pub fn success_break(value: WrappedValue<'tree>) -> Self {
        Self {
            break_value: Some(value),
            ..Default::default()
        }
    }

    pub fn success_return(value: WrappedValue<'tree>) -> Self {
        Self {
            return_value: Some(value),
            ..Default::default()
        }
    }

    pub fn should_return(&self) -> bool {
        self.should_continue || self.break_value.is_some() || self.return_value.is_some()
    }

    pub fn take_value(self) -> WrappedValue<'tree> {
        self.value.unwrap_or_else(|| Value::Null.wrapped())
    }
}
