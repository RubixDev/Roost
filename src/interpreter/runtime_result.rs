use super::value::Value;

#[derive(Debug)]
pub struct RuntimeResult {
    pub should_continue: bool,
    pub should_break: bool,
    pub return_value: Option<Value>,
    pub parent_value: Option<Value>,
    pub value: Option<Value>,
}

impl RuntimeResult {
    pub fn new() -> RuntimeResult {
        return RuntimeResult {
            should_continue: false,
            should_break: false,
            return_value: None,
            parent_value: None,
            value: None,
        };
    }

    pub fn success(&mut self, value: Option<Value>) {
        self.reset();
        self.value = value;
    }

    pub fn success_continue(&mut self) {
        self.reset();
        self.should_continue = true;
    }

    pub fn success_break(&mut self) {
        self.reset();
        self.should_break = true;
    }

    pub fn success_return(&mut self, value: Option<Value>) {
        self.reset();
        self.return_value = value;
    }

    pub fn register(&mut self, result: RuntimeResult) {
        self.should_continue = result.should_continue;
        self.should_break    = result.should_break;
        self.return_value    = result.return_value;
        self.parent_value    = result.parent_value;
        self.value           = result.value;
    }

    pub fn should_return(&self) -> bool {
        return self.should_continue || self.should_break || self.return_value != None;
    }

    fn reset(&mut self) {
        self.should_continue = false;
        self.should_break    = false;
        self.return_value    = None;
        self.parent_value    = self.value.clone();
        self.value           = None;
    }
}
