use crate::{Environment, Expr};
use std::{fmt, rc::Rc};

/// A reduced exact fraction owned by the language runtime.
/// Скорочений точний дріб, яким володіє runtime мови.
/// Ein gekürzter exakter Bruch im Besitz der Sprachlaufzeit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rational {
    pub numerator: i64,
    pub denominator: i64,
}

impl Rational {
    pub fn new(numerator: i64, denominator: i64) -> Option<Self> {
        Self::from_i128(i128::from(numerator), i128::from(denominator))
    }

    fn from_i128(mut numerator: i128, mut denominator: i128) -> Option<Self> {
        if denominator == 0 {
            return None;
        }
        if denominator < 0 {
            numerator = -numerator;
            denominator = -denominator;
        }
        let divisor = gcd(numerator.unsigned_abs(), denominator as u128) as i128;
        let numerator = i64::try_from(numerator / divisor).ok()?;
        let denominator = i64::try_from(denominator / divisor).ok()?;
        Some(Self { numerator, denominator })
    }

    pub fn integer(value: i64) -> Self {
        Self { numerator: value, denominator: 1 }
    }

    pub fn checked_div(self, divisor: Self) -> Option<Self> {
        if divisor.numerator == 0 {
            return None;
        }
        let numerator = i128::from(self.numerator) * i128::from(divisor.denominator);
        let denominator = i128::from(self.denominator) * i128::from(divisor.numerator);
        Self::from_i128(numerator, denominator)
    }
}

fn gcd(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}

/// A closure keeps executable forms together with their lexical environment.
/// Замикання зберігає виконувані форми разом із їхнім лексичним середовищем.
/// Eine Closure bewahrt ausführbare Formen zusammen mit ihrer lexikalischen Umgebung auf.
#[derive(Clone, Debug)]
pub struct Closure {
    pub(crate) parameters: Vec<String>,
    pub(crate) body: Vec<Expr>,
    pub(crate) environment: Environment,
}

/// Runtime data is independent of the parser and any host representation.
/// Дані виконання не залежать від парсера та представлення у хост-системі.
/// Laufzeitdaten sind unabhängig vom Parser und von jeder Host-Darstellung.
#[derive(Clone, Debug)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Rational(Rational),
    String(String),
    Symbol(String),
    Pair(Box<Value>, Box<Value>),
    Closure(Rc<Closure>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(left), Value::Bool(right)) => left == right,
            (Value::Number(left), Value::Number(right)) => left == right,
            (Value::Rational(left), Value::Rational(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            (Value::Symbol(left), Value::Symbol(right)) => left == right,
            (Value::Pair(left_head, left_tail), Value::Pair(right_head, right_tail)) => {
                left_head == right_head && left_tail == right_tail
            }
            // Functions have identity: two separately created closures are not equal.
            // Функції мають ідентичність: два окремо створені замикання не є рівними.
            // Funktionen besitzen Identität: Zwei getrennt erzeugte Closures sind nicht gleich.
            (Value::Closure(left), Value::Closure(right)) => Rc::ptr_eq(left, right),
            _ => false,
        }
    }
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
            Value::Rational(number) => write!(formatter, "{}/{}", number.numerator, number.denominator),
            Value::String(value) => write!(formatter, "\"{value}\""),
            Value::Symbol(symbol) => write!(formatter, "{symbol}"),
            Value::Pair(_, _) => write_pair(formatter, self),
            Value::Closure(_) => write!(formatter, "<lambda>"),
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
