use std::fs::File;
use crate::bytecode::ByteCode;
use crate::lex::{Lex, Token};
use crate::value::Value;

#[derive(Debug)]
pub struct ParseProto {
    pub constants: Vec::<Value>,
    // 常量表
    pub byte_codes: Vec::<ByteCode>, // 字节码序列
}

pub fn load(input: File) -> ParseProto {
    let mut constants = Vec::new(); // 常量表
    let mut byte_codes = Vec::new(); // 字节码序列
    let mut lex = Lex::new(input); // 词法分析器

    loop {
        match lex.next() {
/*
遇到Name，认为是语句开始：
*/
            Token::Name(name) => { // `Name LiteralString` as function call
/*
1. 把Name作为全局变量，存入常量表中；
2. 生成GetGlobal字节码，把根据名字把全局变量加载到栈上。第1个参数是目标栈索引，由于我们目前只支持函数调用语言，栈只用来函数调用，所以函数一定是在0的位置；第2个参数是全局变量名在全局变量中的索引；
3. 读取下一个Token，并预期是字符串常量，否则panic；
4. 把字符串常量加入到常量表中；
5. 生成LoadConst字节码，把常量加载到栈上。第1个参数是目标栈索引，排在函数的后面，为1；第2个参数是常量在常量表中的索引；
6. 准备好了函数和参数，就可以生成Call字节码，调用函数。目前2个参数，分别是函数位置和参数个数，分别固定为0和1。
*/
                constants.push(Value::String(name));
                byte_codes.push(ByteCode::GetGlobal(0, (constants.len() - 1) as u8));

                if let Token::String(s) = lex.next() {
                    constants.push(Value::String(s));
                    byte_codes.push(ByteCode::LoadConst(1, (constants.len() - 1) as u8));
                    byte_codes.push(ByteCode::Call(0, 1));
                } else {
                    panic!("expected string");
                }
            }
/*
遇到Eos，退出循环。
*/
            Token::Eos => break,
/*
遇到其他Token（目前只能是Token::String类型），则panic。
*/
            t => panic!("unexpected token: {t:?}"),
        }
    }

    dbg!(&constants); // 打印常量表
    dbg!(&byte_codes); // 打印字节码序列
    ParseProto { constants, byte_codes } // 返回ParseProto
}