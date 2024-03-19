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
                    if self.lex.peek() == &Token::Assign { // Name = exp 全局变量命名
                        self.assignment(name);
                    } else {
                        self.function_call(name); // Name(exp), Name String 函数调用
                    }
                }
                Token::Local => { self.local() } // local Name = exp 局部变量命名
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

        if let Some(i) = self.get_local(&var) { // 检查左值是否是局部变量
            // local variable
            self.load_exp(i);
        } else {
            // global variable
            let dst = self.add_const(Value::String(var)) as u8; // 将变量名加载到常量表中，并返回索引

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