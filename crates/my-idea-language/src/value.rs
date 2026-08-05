use std::fmt;

/// Runtime data is independent of the parser and any host representation.
/// Дані виконання не залежать від парсера та представлення у хост-системі.
/// Laufzeitdaten sind unabhängig vom Parser und von jeder Host-Darstellung.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Symbol(String),
    Pair(Box<Value>, Box<Value>),
}

impl Value {
    pub fn list(values: impl IntoIterator<Item = Value>) -> Self {
        values
            .into_iter()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .fold(Value::Nil, |tail, head| {
                Value::Pair(Box::new(head), Box::new(tail))
            })
    }

    pub fn is_atom(&self) -> bool {
        !matches!(self, Value::Pair(_, _))
    }

    pub fn is_truthy(&self) -> bool {
        !matches!(self, Value::Nil | Value::Bool(false))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(formatter, "()"),
            Value::Bool(true) => write!(formatter, "t"),
            Value::Bool(false) => write!(formatter, "()"),
            Value::Number(number) => write!(formatter, "{number}"),
            Value::String(value) => write!(formatter, "\"{value}\""),
            Value::Symbol(symbol) => write!(formatter, "{symbol}"),
            Value::Pair(_, _) => write_pair(formatter, self),
        }
    }
}

fn write_pair(formatter: &mut fmt::Formatter<'_>, value: &Value) -> fmt::Result {
    write!(formatter, "(")?;
    let mut current = value;
    let mut first = true;
    loop {
        match current {
            Value::Pair(head, tail) => {
                if !first {
                    write!(formatter, " ")?;
                }
                write!(formatter, "{head}")?;
                current = tail;
                first = false;
            }
            Value::Nil => return write!(formatter, ")"),
            tail => return write!(formatter, " . {tail})"),
        }
    }
}
