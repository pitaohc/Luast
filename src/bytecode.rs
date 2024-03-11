
/// 字节码
/// 枚举字节码的类型
/// 目前包含三种字节码
/// 1. 获取全局变量
/// 2. 加载常量
/// 3. 调用函数
#[derive(Debug)]
pub enum ByteCode {
    GetGlobal(u8, u8),
    // 获取全局变量
    LoadConst(u8, u8),
    // 加载常量
    Call(u8, u8), // 调用函数
}
