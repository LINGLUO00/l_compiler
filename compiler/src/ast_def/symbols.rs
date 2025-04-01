use koopa::ir::TypeKind;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Ident{
    pub ident:String,
}


pub struct FuncType {
    //函数返回值类型
    //pub ret_ty:TypeKind
    pub ret_ty:TypeKind
}
impl Debug for FuncType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ret_ty)
    }
}