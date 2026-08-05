use crate::Value;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Clone, Debug)]
pub struct Environment(Rc<RefCell<Frame>>);

#[derive(Debug)]
struct Frame {
    values: HashMap<String, Value>,
    parent: Option<Environment>,
}

impl Environment {
    pub fn root() -> Self {
        let environment = Self(Rc::new(RefCell::new(Frame {
            values: HashMap::new(),
            parent: None,
        })));
        environment.define("t", Value::Bool(true));
        environment
    }

    /// A child frame is the future lexical boundary captured by a closure.
    /// Дочірній фрейм стане майбутньою лексичною межею, яку зберігатиме замикання.
    /// Ein untergeordneter Frame ist die künftige lexikalische Grenze einer Closure.
    pub fn child(&self) -> Self {
        Self(Rc::new(RefCell::new(Frame {
            values: HashMap::new(),
            parent: Some(self.clone()),
        })))
    }

    pub fn define(&self, name: impl Into<String>, value: Value) {
        self.0.borrow_mut().values.insert(name.into(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        let frame = self.0.borrow();
        frame
            .values
            .get(name)
            .cloned()
            .or_else(|| frame.parent.as_ref()?.get(name))
    }
}

#[derive(Clone, Debug)]
pub struct Session {
    pub environment: Environment,
    pub output: Vec<String>,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            environment: Environment::root(),
            output: Vec::new(),
        }
    }
}
