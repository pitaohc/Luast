use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::mem;

// ANCHOR: token
#[derive(Debug, PartialEq)]
pub enum Token {
    // keywords
    And,    Break,  Do,     Else,   Elseif, End,
    False,  For,    Function, Goto, If,     In,
    Local,  Nil,    Not,    Or,     Repeat, Return,
    Then,   True,   Until,  While,

    // +       -       *       /       %       ^       #
    Add,    Sub,    Mul,    Div,    Mod,    Pow,    Len,
    // &       ~       |       <<      >>      //
    BitAnd, BitXor, BitOr,  ShiftL, ShiftR, Idiv,
    // ==       ~=     <=      >=      <       >        =
    Equal,  NotEq,  LesEq,  GreEq,  Less,   Greater, Assign,
    // (       )       {       }       [       ]       ::
    ParL,   ParR,   CurlyL, CurlyR, SqurL,  SqurR,  DoubColon,
    // ;               :       ,       .       ..      ...
    SemiColon,      Colon,  Comma,  Dot,    Concat, Dots,

    // constant values
    Integer(i64),
    Float(f64),
    String(String),

    // name of variables or table keys
    Name(String),

    // end
    Eos,
}
// ANCHOR_END: token

#[derive(Debug)]
// ANCHOR: lex
pub struct Lex {
    input: File,
    ahead: Token,
}
// ANCHOR_END: lex

impl Lex {
    pub fn new(input: File) -> Self {
        Lex {
            input,
            ahead: Token::Eos, // 上一个Token
        }
    }

    // ANCHOR: peek_next

    /// 获得下一个Token
    pub fn next(&mut self) -> Token {
        if self.ahead == Token::Eos {
            // 如果没有前一个Token，就读取下一个Token
            self.do_next()
        } else {
            mem::replace(&mut self.ahead, Token::Eos) // 如果有前一个Token，就返回前一个Token
            /*
            mem::replace功能是替换掉一个变量的值，并返回原来的值
            */
        }
    }

    pub fn peek(&mut self) -> &Token {
        if self.ahead == Token::Eos {
            self.ahead = self.do_next();
        }
        &self.ahead
    }
    // ANCHOR_END: peek_next

    fn do_next(&mut self) -> Token {
        let ch = self.read_char(); // 读取一个字符
        match ch { // 字符匹配
            '\n' | '\r' | '\t' | ' ' => self.do_next(), // 空白字符
            '+' => Token::Add,
            '*' => Token::Mul,
            '%' => Token::Mod,
            '^' => Token::Pow,
            '#' => Token::Len,
            '&' => Token::BitAnd,
            '|' => Token::BitOr,
            '(' => Token::ParL,
            ')' => Token::ParR,
            '{' => Token::CurlyL,
            '}' => Token::CurlyR,
            '[' => Token::SqurL,
            ']' => Token::SqurR,
            ';' => Token::SemiColon,
            ',' => Token::Comma,
            '/' => self.check_ahead('/', Token::Idiv, Token::Div), // 除法或整除，需要再读一个字符
            '=' => self.check_ahead('=', Token::Equal, Token::Assign), // 等于或赋值，需要再读一个字符
            '~' => self.check_ahead('=', Token::NotEq, Token::BitXor), // 不等于或按位异或，需要再读一个字符
            ':' => self.check_ahead(':', Token::DoubColon, Token::Colon), // 双冒号或冒号，需要再读一个字符
            '<' => self.check_ahead2('=', Token::LesEq, '<', Token::ShiftL, Token::Less), // 小于或小于等于或左移，需要再读2个字符
            '>' => self.check_ahead2('=', Token::GreEq, '>', Token::ShiftR, Token::Greater), // 大于或大于等于或右移，需要再读2个字符
            '\'' | '"' => self.read_string(ch), // 字符串 '' 或 ""
            '.' => match self.read_char() { // 点 . 或 .. 或 ...
                '.' => { // ..
                    if self.read_char() == '.' { // ...
                        Token::Dots
                    } else {
                        self.putback_char(); // ..
                        Token::Concat
                    }
                }
                '0'..='9' => { // 浮点数
                    self.putback_char();
                    self.read_number_fraction(0)
                }
                _ => { // .
                    self.putback_char();
                    Token::Dot
                }
            },
            '-' => { // 减号或注释
                if self.read_char() == '-' { // -- 注释
                    self.read_comment();
                    self.do_next()
                } else {
                    self.putback_char();
                    Token::Sub // 减号
                }
            }
            '0'..='9' => self.read_number(ch), // 数字
            'A'..='Z' | 'a'..='z' | '_' => self.read_name(ch), // 标识符
            '\0' => Token::Eos, // 文件结束
            _ => panic!("invalid char {ch}"), // 异常Token
        }
    }

    #[allow(clippy::unused_io_amount)]
    fn read_char(&mut self) -> char {
        let mut buf: [u8; 1] = [0];
        self.input.read(&mut buf).unwrap();
        buf[0] as char
    }

    /// 回退一个字符
    fn putback_char(&mut self) {
        /*
        input 是打开的源代码文件
        seek是文件指针的移动
        SeekFrom::Current(0) 表示从当前位置移动0个字节
        -1 表示向前移动一个字节
        seek() 返回的是Result，所以用unwrap()解包
        */
        self.input.seek(SeekFrom::Current(-1)).unwrap();
    }

    fn check_ahead(&mut self, ahead: char, long: Token, short: Token) -> Token {
        if self.read_char() == ahead {
            long
        } else {
            self.putback_char();
            short
        }
    }
    fn check_ahead2(
        &mut self,
        ahead1: char,
        long1: Token,
        ahead2: char,
        long2: Token,
        short: Token,
    ) -> Token {
        let ch = self.read_char();
        if ch == ahead1 {
            long1
        } else if ch == ahead2 {
            long2
        } else {
            self.putback_char();
            short
        }
    }


    /// 读取数字
    /// first 是第一个数字字符，因为要根据第一个字符判断是否是十六进制
    /// 读取整数和浮点数
    fn read_number(&mut self, first: char) -> Token {
        // 十六进制，判断条件：第一个字符是0，第二个字符是x或X
        if first == '0' {
            let second = self.read_char();
            if second == 'x' || second == 'X' {
                return self.read_heximal();
            }
            self.putback_char();
        }

        // 十进制，读取整数部分
        let mut n = char::to_digit(first, 10).unwrap() as i64;
        loop {
            let ch = self.read_char();
            if let Some(d) = char::to_digit(ch, 10) {
                n = n * 10 + d as i64;
            } else if ch == '.' {
                return self.read_number_fraction(n); // 读取小数部分
            } else if ch == 'e' || ch == 'E' {
                return self.read_number_exp(n as f64); // 读取科学计数法
            } else {
                self.putback_char();
                break;
            }
        }

        // check following
        let fch = self.read_char();
        if fch.is_alphabetic() || fch == '.' {
            panic!("malformat number");
        } else {
            self.putback_char();
        }

        Token::Integer(n)
    }

    /// 读取浮点数
    fn read_number_fraction(&mut self, i: i64) -> Token {
        let mut n: i64 = 0;
        let mut x: f64 = 1.0;
        loop {
            let ch = self.read_char();
            if let Some(d) = char::to_digit(ch, 10) {
                n = n * 10 + d as i64;
                x *= 10.0;
            } else {
                self.putback_char();
                break;
            }
        }
        Token::Float(i as f64 + n as f64 / x) // 返回一个浮点数
    }

    /// 读取科学计数法
    fn read_number_exp(&mut self, _: f64) -> Token {
        todo!("lex number exp")
    }

    /// 读取十六进制
    fn read_heximal(&mut self) -> Token {
        todo!("lex heximal")
    }


    /// 读取字符串
    fn read_string(&mut self, quote: char) -> Token {
        let mut s = String::new();
        loop {
            match self.read_char() {
                '\n' | '\0' => panic!("unfinished string"),
                '\\' => todo!("escape"),
                ch if ch == quote => break,
                ch => s.push(ch),
            }
        }
        Token::String(s)
    }

    /// 读取标识符
    fn read_name(&mut self, first: char) -> Token {
        let mut s = first.to_string();

        loop {
            let ch = self.read_char();
            if ch.is_alphanumeric() || ch == '_' {
                s.push(ch);
            } else {
                self.putback_char();
                break;
            }
        }

        match &s as &str {
            // TODO optimize by hash
            "and" => Token::And,
            "break" => Token::Break,
            "do" => Token::Do,
            "else" => Token::Else,
            "elseif" => Token::Elseif,
            "end" => Token::End,
            "false" => Token::False,
            "for" => Token::For,
            "function" => Token::Function,
            "goto" => Token::Goto,
            "if" => Token::If,
            "in" => Token::In,
            "local" => Token::Local,
            "nil" => Token::Nil,
            "not" => Token::Not,
            "or" => Token::Or,
            "repeat" => Token::Repeat,
            "return" => Token::Return,
            "then" => Token::Then,
            "true" => Token::True,
            "until" => Token::Until,
            "while" => Token::While,
            _ => Token::Name(s),
        }
    }


    /// 读取注释
    /// '--' 已经读取
    fn read_comment(&mut self) {
        match self.read_char() {
            '[' => todo!("long comment"), // 长注释
            _ => {
                // line comment
                loop {
                    let ch = self.read_char();
                    if ch == '\n' || ch == '\0' {
                        break;
                    }
                }
            }
        }
    }
}
