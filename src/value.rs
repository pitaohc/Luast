use std::fmt;
use crate::vm::ExeState;

#[derive(Clone)]
pub enum Value {
    Nil,
    String(String),
    Function(fn(&mut ExeState) -> i32),
}


impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "Nil"),
            Value::String(s) => write!(f, "{}", s),
            Value::Function(_) => write!(f, "Function"),
        }
    }
}