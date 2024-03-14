#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::File;
    use Luast::lex::{Lex, Token};
    use Luast::lex::Token::*;
    use Luast::parse;
    use Luast::value::Value;

    #[test]
    fn simple_print() {
        let file = File::open("test_lua/hello.lua").unwrap();
        /*
        unwrap 如果Result是Ok，则返回Ok中的值，如果Result是Err，则直接panic
        */
        /*
        通过语法分析获得字节码
        */
        let mut lex = Lex::new(file);
        let mut res: Vec<Token> = Vec::new();
        loop {
            let token = lex.next();
            print!("{:?}\n", token);
            if token == Token::Eos {
                break;
            }
            res.push(token);
        }

        let ans = vec![
            Token::Name("print".to_string()),
            Token::String("hello, world!".to_string()), //line 1

        ];
        assert_eq!(res.len(), ans.len()); // 长度相等
        for id in 0..res.len() {
            let token_res = &res[id];
            let token_ans = &ans[id];
            assert_eq!(token_res, token_ans);
        }
    }

    #[test]
    fn tokens_print() {
        let file = File::open("test_lua/tokens.lua").unwrap();

        let mut lex = Lex::new(file);
        let mut res: Vec<Token> = Vec::new();
        loop {
            let token = lex.next();
            print!("{:?}\n", token);
            if token == Token::Eos {
                break;
            }
            res.push(token);
        }

        let ans = vec![
            Token::Name("print".to_string()),
            Token::String("hello".to_string()), //line 1

            Token::Name("print".to_string()),
            Token::ParL,
            Token::Integer(123),
            Token::ParR,
            Token::Name("print".to_string()),
            Token::ParL,
            Token::Float(3.14),
            Token::ParR,
            Token::Name("print".to_string()),
            Token::ParL,
            Token::True,
            Token::ParR,
        ];
        assert_eq!(res.len(), ans.len()); // 长度相等
        for id in 0..res.len() {
            let token_res = &res[id];
            let token_ans = &ans[id];
            assert_eq!(token_res, token_ans);
        }
    }

    #[test]
    fn all_tokens_print() {
        let ans = vec![
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
            Integer(111),
            Float(0.123),
            Token::String("hello".to_string()),

            // name of variables or table keys
            Name("hello".to_string()),

        ];


        let file = File::open("test_lua/all_tokens.lua").unwrap();

        let mut lex = Lex::new(file);
        let mut res: Vec<Token> = Vec::new();
        loop {
            let token = lex.next();
            print!("{:?}\n", token);
            if token == Token::Eos {
                break;
            }
            res.push(token);
        }
        assert_eq!(res.len(), ans.len()); // 长度相等
        for id in 0..res.len() {
            let token_res = &res[id];
            let token_ans = &ans[id];
            assert_eq!(token_res, token_ans);
        }
    }
}