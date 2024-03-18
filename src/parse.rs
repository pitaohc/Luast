use std::fs::File;
use crate::bytecode::ByteCode;
use crate::lex::{Lex, Token};
use crate::value::Value;

///
/// 语法分析器
/// # Arguments
/// * `constants` - 常量表
/// * `byte_codes` - 字节码序列
/// * `locals` - 局部变量表
/// * `lex` - 词法分析器
#[derive(Debug)]
pub struct ParseProto {
    pub constants: Vec<Value>,
    pub byte_codes: Vec<ByteCode>,
    locals: Vec<String>,
    lex: Lex,
}

impl ParseProto {
    pub fn load(input: File) -> ParseProto {
        let mut proto = ParseProto {
            constants: Vec::new(),
            byte_codes: Vec::new(),
            locals: Vec::new(),
            lex: Lex::new(input),
        };
        proto.chunk();
        println!("constants: {:?}", &proto.constants);
        println!("byte_codes:");
        for c in proto.byte_codes.iter() {
            println!("  {:?}", c);
        }
        proto
    }

    /// 解析token流
    fn chunk(&mut self) {
        loop {
            match self.lex.next() {
                Token::Name(name) => {
                    if self.lex.peek() == &Token::Assign { // 全局变量命名
                        self.assignment(name);
                    } else {
                        self.function_call(name); //函数调用
                    }
                }
                Token::Local => { self.local() }
                Token::Eos => break,
                t => panic!("unexpected token: {t:?}"),
            }
        }
    }

    /// Name LiteralString
    /// Name ( exp )
    fn function_call(&mut self, name: String) {
        let ifunc = self.locals.len();
        let iarg = ifunc + 1;

        // function, variable
        let code = self.load_var(ifunc, name);
        self.byte_codes.push(code);
        // argument, (exp) or "string", only one argument
        // TODO 修改函数读取为多参数
        match self.lex.next() {
            Token::ParL => { // '(' 开始
                self.load_exp(iarg); // 加载表达式
                if self.lex.next() != Token::ParR { // ')' 结束
                    panic!("expected `)`");
                }
            }
            Token::String(s) => {
                let code = self.load_const(iarg, Value::String(s));
                self.byte_codes.push(code);
            }
            t => panic!("unexpected token: {t:?}"),
        }
        // 添加调用函数的字节码
        self.byte_codes.push(ByteCode::Call(ifunc as u8, 1));
    }
    /// local Name = exp
    ///
    /// 遇到local，定义局部变量,
    /// 将常量或字面值加载到栈中
    fn local(&mut self) {
        let var = if let Token::Name(var) = self.lex.next() {
            var
        } else {
            panic!("expected variable");
        };

        if Token::Assign != self.lex.next() {
            panic!("expected '='")
        }
        self.load_exp(self.locals.len());

        // add to locals after load_exp()
        self.locals.push(var);
    }

    /// 赋值操作
    fn assignment(&mut self, var: String) {
        self.lex.next(); // `=`

        if let Some(i) = self.get_local(&var) {
            // local variable
            self.load_exp(i);
        } else {
            // global variable
            let dst = self.add_const(Value::String(var)) as u8;

            let code = match self.lex.next() {
                // from const values
                Token::Nil => ByteCode::SetGlobalConst(dst, self.add_const(Value::Nil) as u8),
                Token::True => ByteCode::SetGlobalConst(dst, self.add_const(Value::Boolean(true)) as u8),
                Token::False => ByteCode::SetGlobalConst(dst, self.add_const(Value::Boolean(false)) as u8),
                Token::Integer(i) => ByteCode::SetGlobalConst(dst, self.add_const(Value::Integer(i)) as u8),
                Token::Float(f) => ByteCode::SetGlobalConst(dst, self.add_const(Value::Float(f)) as u8),
                Token::String(s) => ByteCode::SetGlobalConst(dst, self.add_const(Value::String(s)) as u8),

                // from variable
                Token::Name(var) =>
                    if let Some(i) = self.get_local(&var) {
                        // local variable
                        ByteCode::SetGlobal(dst, i as u8)
                    } else {
                        // global variable
                        ByteCode::SetGlobalGlobal(dst, self.add_const(Value::String(var)) as u8)
                    }

                _ => panic!("invalid argument"),
            };
            self.byte_codes.push(code);
        }
    }


    /// 将值c加入常量表，并返回索引。
    /// 如果常量表中已经存在该常量，就返回该常量的索引。
    ///
    /// # 参数
    /// * `c` - 常量
    /// # 返回值
    /// * `usize` - 常量的索引
    fn add_const(&mut self, c: Value) -> usize {
        let constants = &mut self.constants;
        constants.iter().position(|x| x == &c).unwrap_or_else(|| {
            constants.push(c);
            constants.len() - 1
        })
    }


    /// 生成加载常量的字节码，
    /// 将常量c加载到指定栈位置。
    fn load_const(&mut self, dst: usize, c: Value) -> ByteCode {
        let ic = self.add_const(c); //获得常量的在常量表中的索引
        ByteCode::LoadConst(dst as u8, ic as u16) //将加载常量的字节码加入字节码序列
    }


    /// 加载变量到指定位置
    fn load_var(&mut self, dst: usize, name: String) -> ByteCode {
        /*
        如果在栈中找到变量，就将变量的索引加载到指定位置
        如果找不到，就将变量名加载到常量表中
        */
        if let Some(i) = self.locals.iter().position(|x| *x == name) {
            ByteCode::Move(dst as u8, i as u8) // 将栈中i位置的值复制到dst位置
        } else {
            let ic = self.add_const(Value::String(name));
            ByteCode::GetGlobal(dst as u8, ic as u8) // 将常量表中ic位置的值加载到dst位置
        }
    }


    /// 获取局部变量索引
    fn get_local(&self, name: &str) -> Option<usize> {
        /*
        locals.iter() 返回一个迭代器
        rposition() 从右侧搜索，返回第一个满足条件的元素的索引
        */
        self.locals.iter().rposition(|x| x == name)
    }


    /// 添加加载表达式的字节码
    ///
    /// # 参数
    /// * `dst` - 目标栈索引
    fn load_exp(&mut self, dst: usize) {
        let code = match self.lex.next() {
            Token::Name(var) => self.load_var(dst, var),
            Token::Nil => ByteCode::LoadNil(dst as u8),
            Token::True => ByteCode::LoadBool(dst as u8, true),
            Token::False => ByteCode::LoadBool(dst as u8, false),
            Token::Integer(i) =>
                if let Ok(ii) = i16::try_from(i) {
                    ByteCode::LoadInt(dst as u8, ii) // 如果整数是i16类型，就直接把数字放入字节码序列
                } else {
                    self.load_const(dst, Value::Integer(i)) // 如果超过i16的表示范围，就从常量表中加载
                },
            Token::Float(f) => self.load_const(dst, Value::Float(f)),
            Token::String(s) => self.load_const(dst, Value::String(s)),
            _ => panic!("invalid argument"),
        };
        self.byte_codes.push(code);
    }
}


// /// load会读取一个lua源代码文件，调用Lex得到Token流。
// /// 根据Token流生成常量表和字节码序列。
// /// 最后返回ParseProto(包含了常量表和字节码序列)。
// ///
// /// # 参数
// /// * `input` - 文件
// /// # 返回值
// /// * `ParseProto` - 语法解析器结构
// pub fn load(input: File) -> ParseProto {
//     let mut constants = Vec::new(); // 常量表
//     let mut byte_codes = Vec::new(); // 字节码序列
//     let mut lex = Lex::new(input); // 词法分析器
//     let mut locals = Vec::new(); // 局部变量表
//     loop {
//         match lex.next() {
//             /*
//             遇到Name，认为是语句开始：
//             - name "string"
//             - name (exp)
//             */
//             Token::Name(name) => { // `Name LiteralString` as function call
//                 // function, global variable only
//                 let ic: usize = add_const(&mut constants, Value::String(name)); // 将Name加入常量表
//                 byte_codes.push(ByteCode::GetGlobal(0, ic as u8)); // 将Name的索引加入字节码序列,目前为函数名
//                 // argument, (var) or "string"
//                 match lex.next() {
//                     // 合法的两种情况：左括号和字符串
//                     Token::ParL => {
//                         let code = match lex.next() {
//                             Token::Nil => ByteCode::LoadNil(1),
//                             Token::True => ByteCode::LoadBool(1, true),
//                             Token::False => ByteCode::LoadBool(1, false),
//                             Token::Integer(i) => {
//                                 // 如果整数是i16类型，就直接把数字放入字节码序列
//                                 // 如果超过i16的表示范围，就从常量表中加载
//                                 if let Ok(ii) = i16::try_from(i) {
//                                     ByteCode::LoadInt(1, ii)
//                                 } else {
//                                     load_const(&mut constants, 1, Value::Integer(i))
//                                 }
//                             }
//                             Token::Float(f) => load_const(&mut constants, 1, Value::Float(f)),
//                             Token::String(s) => load_const(&mut constants, 1, Value::String(s)),
//                             t => panic!("unexpected token: {t:?}"),
//                         };
//                         byte_codes.push(code);
//                         if lex.next() != Token::ParR {
//                             panic!("expected `)`");
//                         }
//                     }
//                     Token::String(s) => {
//                         //生成从常量表中加载字符串的字节码，并压入字节码序列（栈）中
//                         let code = load_const(&mut constants, 1, Value::String(s));
//                         byte_codes.push(code);
//                     }
//
//                     // 非法情况，引发panic，并输出错误信息
//                     t => panic!("unexpected token: {t:?}"),
//                 }
//                 byte_codes.push(ByteCode::Call(0, 1));
//             }
//             /*
//             遇到local，定义局部变量
//             将常量或字面值加载到栈中
//             */
//             Token::Local => { // local name = exp
//                 let var = if let Token::Name(var) = lex.next() {
//                     var
//                 } else {
//                     panic!("expected variable");
//                 };
//
//                 if Token::Assign != lex.next() {
//                     panic!("expected '='")
//                 }
//                 let size = locals.len();
//                 load_exp(&mut byte_codes, &mut constants, &mut locals, lex.next(), size);
//
//                 // add to locals after load_exp()
//                 locals.push(var);
//             }
//             /*
//             遇到Eos，退出循环。
//             */
//             Token::Eos => break,
//             /*
//             遇到其他Token（目前只能是Token::String类型），则panic。
//             */
//             t => panic!("unexpected token: {t:?}"),
//         }
//     }
//
//     dbg!(&constants); // 打印常量表
//
//     dbg!(&byte_codes); // 打印字节码序列
//
//     ParseProto { constants, byte_codes, locals,lex } // 返回ParseProto
// }
//
// fn load_const(constants: &mut Vec<Value>, dst: u8, v: Value) -> ByteCode {
//     let ic = add_const(constants, v);
//     ByteCode::LoadConst(dst, ic as u16)
// }
//
// fn add_const(constants: &mut Vec<Value>, c: Value) -> usize {
//     // 如果常量表中已经存在该常量，就返回该常量的索引
//     // iter 返回一个迭代器，position 返回第一个满足条件的元素的索引
//     // position 的参数是一个闭包，闭包的参数x是迭代器的元素，返回值是一个bool
//     // *x 代表解引用，即取出迭代器的元素
//     if let Some(i) = constants.iter().position(|x| *x == c) {
//         i
//     } else {
//         constants.push(c);
//         constants.len() - 1
//     }
// }
//
// pub fn get_const(constants: &Vec<Value>, id: usize) -> &Value {
//     constants.get(id).unwrap()
// }
//
//
// /// 加载表达式
// ///
// /// # 参数
// /// * `byte_codes` - 字节码序列
// pub fn load_exp(byte_codes: &mut Vec<ByteCode>, constants: &mut Vec<Value>,
//                 locals: &Vec<String>, token: Token, dst: usize) {
//     let code = match token {
//         Token::Name(var) => load_var(constants, locals, dst, var),
//         _ => panic!("invalid argument"),
//     };
//     byte_codes.push(code);
// }
//
//
// /// 加载变量
// /// # 参数
// /// * `constants` - 常量表
// /// * `locals` - 局部变量表
// /// * `dst` - 目标栈索引
// /// * `name` - 变量名
// pub fn load_var(constants: &mut Vec<Value>, locals: &Vec<String>, dst: usize, name: String) -> ByteCode {
//     if let Some(i) = locals.iter().position(|x| *x == name) {
//         ByteCode::Move(dst as u8, i as u8)
//     } else {
//         let ic = add_const(constants, Value::String(name));
//         ByteCode::GetGlobal(dst as u8, ic as u8)
//     }
// }