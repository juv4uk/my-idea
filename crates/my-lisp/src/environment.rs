use crate::Value;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

/// **Known Risk:** Dropping a deeply nested `Environment` chain (thousands of levels)
/// could cause a stack overflow because `Rc<RefCell<Frame>>` uses Rust's recursive `Drop`.
/// This is not currently an issue since we only have one child level from root in most usage,
/// but it could appear if deep nesting of `let` or currying patterns emerges.
#[derive(Clone, Debug)]
pub struct Environment(Rc<RefCell<Frame>>);

#[derive(Debug)]
struct Frame {
    values: HashMap<Rc<str>, Value>,
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

    pub fn define(&self, name: impl Into<Rc<str>>, value: Value) {
        self.0.borrow_mut().values.insert(name.into(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        let mut current = Some(self.clone());
        while let Some(env) = current {
            if let Some(value) = env.0.borrow().values.get(name) {
                return Some(value.clone());
            }
            current = env.0.borrow().parent.clone();
        }
        None
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
