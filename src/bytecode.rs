use std::fmt;
use std::fmt::Formatter;

/// 字节码
/// 枚举字节码的类型
/// 目前包含字节码如下
/// * GetGlobal 把全局变量赋值给栈上值
/// * SetGlobal 把局部变量赋值给全局变量
/// * SetGlobalConst 把常量赋值给全局变量
/// * SetGlobalGlobal 把全局变量赋值给全局变量
/// * LoadConst 加载常量到栈上
/// * LoadNil 加载Nil到栈上
/// * LoadBool 加载Bool到栈上
/// * LoadInt 加载Int到栈上
/// * Move 将栈上值复制到另一个栈上
/// * Call 调用函数
pub enum ByteCode {
    GetGlobal(u8, u8),
    SetGlobal(u8, u8),
    SetGlobalConst(u8, u8),
    // TODO u8?
    SetGlobalGlobal(u8, u8),
    LoadConst(u8, u16),
    LoadNil(u8),
    LoadBool(u8, bool),
    LoadInt(u8, i16),
    Move(u8, u8),
    Call(u8, u8),
}

impl fmt::Display for ByteCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ByteCode::GetGlobal(dst, name) => write!(f, "GetGlobal {} {}", dst, name),
            ByteCode::SetGlobal(dst, name) => write!(f, "SetGlobal {} {}", dst, name),
            ByteCode::SetGlobalConst(dst, name) => write!(f, "SetGlobalConst {} {}", dst, name),
            ByteCode::SetGlobalGlobal(dst, name) => write!(f, "SetGlobalGlobal {} {}", dst, name),
            ByteCode::LoadConst(dst, c) => write!(f, "LoadConst {} {}", dst, c),
            ByteCode::LoadNil(dst) => write!(f, "LoadNil {}", dst),
            ByteCode::LoadBool(dst, b) => write!(f, "LoadBool {} {}", dst, b),
            ByteCode::LoadInt(dst, i) => write!(f, "LoadInt {} {}", dst, i),
            ByteCode::Move(dst, src) => write!(f, "Move {} {}", dst, src),
            ByteCode::Call(func, args) => write!(f, "Call {} {}", func, args),
            _ => write!(f, "Unimplemented"),
        }
    }
}

impl fmt::Debug for ByteCode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_display() {
        let code = ByteCode::GetGlobal(1, 2);
        dbg!(&code);
        assert_eq!(format!("{}", code), "GetGlobal 1 2");
    }
}
