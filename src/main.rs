use std::env;
// 获取命令行参数
use std::fs::File;


mod value;
mod bytecode;
mod lex;
mod parse;
mod vm;

fn main() {
    let args:Vec::<String> = env::args().collect();
    if args.len() != 2{
        println!("Usage: {} script", args[0]);
        return;
    }
    let file = File::open(&args[1]).unwrap();
    /*
    unwrap 如果Result是Ok，则返回Ok中的值，如果Result是Err，则直接panic
    */
    /*
    通过语法分析获得字节码
    */
    let proto = parse::load(file);
    /*
    新建一个虚拟机执行字节码
    &proto 传递proto的引用
    */
    vm::ExeState::new().execute(&proto);
}