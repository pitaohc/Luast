use std::cmp::Ordering;
use std::collections::HashMap;
use crate::bytecode::ByteCode;
use crate::parse::{get_const, ParseProto};
use crate::value::Value;

/// 虚拟机状态
pub struct ExeState {
    globals: HashMap<String, Value>,
    // 全局变量表
    stack: Vec<Value>, // 栈
}

impl ExeState {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        globals.insert(String::from("print"), Value::Function(lib_print));

        ExeState {
            globals,
            stack: Vec::new(),
        }
    }

    pub fn execute(&mut self, proto: &ParseProto) {
        for code in proto.byte_codes.iter() {
            match *code {
                ByteCode::GetGlobal(dst, name) => {

                    let name = get_const(&proto.constants, name as usize);
                    if let Value::String(key) = name {
                        // 模式匹配,如果name是String类型,则将name作为key从全局变量表中取出对应的值
                        let v = self.globals.get(key).unwrap_or(&Value::Nil).clone();
                        self.set_stack(dst, v); // 将取出的值放入栈中
                    } else {
                        panic!("invalid global key: {name:?}");
                    }
                }
                ByteCode::LoadConst(dst, c) => {
                    let v = get_const(&proto.constants, c as usize).clone();
                    self.set_stack(dst, v);
                }
                ByteCode::Call(func, _) => {
                    let func = &self.stack[func as usize];
                    if let Value::Function(f) = func{
                        f(self);
                    }else{
                        panic!("invalid function: {func:?}");
                    }
                }

                ByteCode::LoadNil(dst) => {
                    self.set_stack(dst, Value::Nil);
                }
                ByteCode::LoadBool(dst, b) => {
                    self.set_stack(dst, Value::Boolean(b));
                }
                ByteCode::LoadInt(dst, i) => {
                    self.set_stack(dst, Value::Integer(i as i64));
                }
                ByteCode::Move(dst, src) => {
                    let v = self.stack[src as usize].clone();
                    self.set_stack(dst, v);
                }
                _ => panic!("unimplemented: {:?}", code)
            }
        }
    }
    fn set_stack(&mut self, dst: u8, v: Value) {
        let dst = dst as usize;
        match dst.cmp(&self.stack.len()) {
            Ordering::Equal => self.stack.push(v),
            Ordering::Less => self.stack[dst] = v,
            Ordering::Greater => panic!("fail in set_stack"),
        }
    }
}

// "print" function in Lua's std-lib.
// It supports only 1 argument and assumes the argument is at index:1 on stack.
fn lib_print(state: &mut ExeState) -> i32 {
    println!("{:?}", state.stack[1]);
    0
}