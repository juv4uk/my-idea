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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_predefines_t_as_true() {
        let root = Environment::root();
        assert_eq!(root.get("t"), Some(Value::Bool(true)));
    }

    #[test]
    fn define_then_get_returns_the_value() {
        let root = Environment::root();
        root.define("x", Value::Number(1.0));
        assert_eq!(root.get("x"), Some(Value::Number(1.0)));
    }

    #[test]
    fn get_on_unknown_name_returns_none() {
        let root = Environment::root();
        assert_eq!(root.get("does-not-exist"), None);
    }

    #[test]
    fn child_reads_bindings_from_its_parent() {
        let root = Environment::root();
        root.define("x", Value::Number(1.0));
        let child = root.child();
        assert_eq!(child.get("x"), Some(Value::Number(1.0)));
    }

    #[test]
    fn child_definitions_do_not_leak_into_the_parent() {
        // Lexical scoping requires that a child frame's bindings stay local:
        // a closure's parameters must never become visible outside its call.
        // Лексичний скоуп вимагає, щоб зв’язування дочірнього фрейму лишались
        // локальними: параметри замикання не повинні ставати видимими зовні виклику.
        // Lexikalischer Scope verlangt, dass Bindungen eines Kind-Frames lokal
        // bleiben: Parameter einer Closure dürfen außerhalb ihres Aufrufs nie sichtbar werden.
        let root = Environment::root();
        let child = root.child();
        child.define("local", Value::Number(2.0));
        assert_eq!(root.get("local"), None);
    }

    #[test]
    fn child_binding_shadows_the_parent_without_mutating_it() {
        let root = Environment::root();
        root.define("x", Value::Number(1.0));
        let child = root.child();
        child.define("x", Value::Number(2.0));
        assert_eq!(child.get("x"), Some(Value::Number(2.0)));
        assert_eq!(root.get("x"), Some(Value::Number(1.0)));
    }

    #[test]
    fn redefining_in_the_same_frame_overwrites_the_previous_value() {
        let root = Environment::root();
        root.define("x", Value::Number(1.0));
        root.define("x", Value::Number(2.0));
        assert_eq!(root.get("x"), Some(Value::Number(2.0)));
    }
}
