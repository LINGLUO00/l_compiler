//! AST定义模块入口

pub mod symbols;
pub mod decl;
pub mod stmt;
pub mod expr;

use self::decl::FuncDef;
use self::symbols::FuncType;

#[derive(Debug)]
pub enum CompUnit {
    Default(Vec<Unit>),
}

#[derive(Debug)]
pub enum Unit{
    //声明单元
    Decl(Decl),
    //函数定义单元
    FuncDef(FuncDef),
}
