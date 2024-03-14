use std::fmt;
use crate::vm::ExeState;

#[derive(Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Function(fn(&mut ExeState) -> i32),
    // 增加类型后记得修改fmt::Debug
}


impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "Nil"),
            Value::String(s) => write!(f, "{s}"),
            Value::Function(_) => write!(f, "Function"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Integer(i) => write!(f, "{i}"),
            Value::Float(n) => write!(f, "{n:?}"),
        }
    }
}