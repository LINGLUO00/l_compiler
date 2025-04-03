use std::fmt::Debug;
use koopa::ir::TypeKind;
//TypeKind包括以下类型
//Int32,
//Unit(void),
/// Array (with base type and length).
//Array(Type, usize),
/// Pointer (with base type).
//Pointer(Type),
/// Function (with parameter types and return type).
//Function(Vec<Type>, Type),

#[derive(Debug)]
pub struct Ident{
    pub ident : String,
}

pub struct BaseType{
    pub base_type:TypeKind,
}

impl Debug for BaseType{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.base_type)
    }
}