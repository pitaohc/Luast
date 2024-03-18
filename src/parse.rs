use std::collections::HashMap;
use std::fs::File;
use crate::bytecode::ByteCode;
use crate::lex::{Lex, Token};
use crate::value::Value;

#[derive(Debug)]
pub struct ParseProto {
    pub constants: Vec<Value>, // 常量表

    pub byte_codes: Vec<ByteCode>, // 字节码序列
    locals: Vec<String>, // 局部变量表

}

/// load会读取一个lua源代码文件，调用Lex得到Token流。
/// 根据Token流生成常量表和字节码序列。
/// 最后返回ParseProto(包含了常量表和字节码序列)
pub fn load(input: File) -> ParseProto {
    let mut constants = Vec::new(); // 常量表
    let mut byte_codes = Vec::new(); // 字节码序列
    let mut lex = Lex::new(input); // 词法分析器
    let mut locals = Vec::new(); // 局部变量表
    loop {
        match lex.next() {
            /*
            遇到Name，认为是语句开始：
            */
            Token::Name(name) => { // `Name LiteralString` as function call
                // function, global variable only
                let ic: usize = add_const(&mut constants, Value::String(name)); // 将Name加入常量表
                byte_codes.push(ByteCode::GetGlobal(0, ic as u8)); // 将Name的索引加入字节码序列,目前为函数名
                // argument, (var) or "string"
                match lex.next() {
                    // 合法的两种情况：左括号和字符串
                    Token::ParL => {
                        let code = match lex.next() {
                            Token::Nil => ByteCode::LoadNil(1),
                            Token::True => ByteCode::LoadBool(1, true),
                            Token::False => ByteCode::LoadBool(1, false),
                            Token::Integer(i) => {
                                // 如果整数是i16类型，就直接把数字放入字节码序列
                                // 如果超过i16的表示范围，就从常量表中加载
                                if let Ok(ii) = i16::try_from(i) {
                                    ByteCode::LoadInt(1, ii)
                                } else {
                                    load_const(&mut constants, 1, Value::Integer(i))
                                }
                            }
                            Token::Float(f) => load_const(&mut constants, 1, Value::Float(f)),
                            Token::String(s) => load_const(&mut constants, 1, Value::String(s)),
                            t => panic!("unexpected token: {t:?}"),
                        };
                        byte_codes.push(code);
                        if lex.next() != Token::ParR {
                            panic!("expected `)`");
                        }
                    }
                    Token::String(s) => {
                        //生成从常量表中加载字符串的字节码，并压入字节码序列（栈）中
                        let code = load_const(&mut constants, 1, Value::String(s));
                        byte_codes.push(code);
                    }

                    // 非法情况，引发panic，并输出错误信息
                    t => panic!("unexpected token: {t:?}"),
                }
                byte_codes.push(ByteCode::Call(0, 1));
            }
            /*
            遇到local，定义局部变量
            */
            Token::Local=>{ // local name = exp
                let var = if let Token::Name(var) = lex.next(){
                    var
                }else{
                    panic!("expected variable");
                };

                if Token::Assign != lex.next(){
                    panic!("expected '='")
                }
                load_exp(&mut byte_codes, &mut constants, lex.next(), locals.len());

                // add to locals after load_exp()
                locals.push(var);
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

    ParseProto { constants, byte_codes,locals } // 返回ParseProto
}

fn load_const(constants: &mut Vec<Value>, dst: u8, v: Value) -> ByteCode {
    let ic = add_const(constants, v);
    ByteCode::LoadConst(dst, ic as u16)
}

fn add_const(constants: &mut Vec<Value>, c: Value) -> usize {
    // 如果常量表中已经存在该常量，就返回该常量的索引
    // iter 返回一个迭代器，position 返回第一个满足条件的元素的索引
    // position 的参数是一个闭包，闭包的参数x是迭代器的元素，返回值是一个bool
    // *x 代表解引用，即取出迭代器的元素
    if let Some(i) = constants.iter().position(|x| *x == c) {
        i
    } else {
        constants.push(c);
        constants.len() - 1
    }
}

pub fn get_const(constants: &Vec<Value>, id: usize) -> &Value {
    constants.get(id).unwrap()
}


/// 加载表达式
///
/// # 参数
/// * `byte_codes` - 字节码序列
pub fn load_exp(byte_codes: &mut Vec<ByteCode>, constants: &mut Vec<Value>,
                locals: &Vec<String>, token: Token, dst: usize){
    let code = match token{
        /// TODO 完成加载其他类型的表达式
        Token::Name(var) => load_var(constants, locals, dst, var),
        _ => panic!("invalid argument"),
    };
    byte_codes.push(code);
}


/// 加载变量
/// # 参数
/// * `constants` - 常量表
/// * `locals` - 局部变量表
/// * `dst` - 目标栈索引
/// * `name` - 变量名
pub fn load_var(constants:&mut Vec<Value>,locals:&Vec<String>,dst:usize,name:String)->ByteCode{
    if let Some(i) = locals.iter().position(|x| *x == name){
        ByteCode::Move(dst as u8, i as u8)
    }else{
        let ic = add_const(constants, Value::String(name));
        ByteCode::GetGlobal(dst as u8, ic as u8)
    }
}