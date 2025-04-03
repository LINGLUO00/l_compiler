pub mod decl;
pub mod symb;
pub mod stmt;
pub mod expr;
use decl::Decl;
use decl::FuncDef;

#[derive(Debug)]
pub struct CompUnit{
    //这里可以增加元数据字段
    // 元数据：模块名（可为空，表示不属于任何模块）
    //pub module_name: Option<String>,
    //实际代码项
    pub items:Vec<Item>,
}

#[derive(Debug)]
pub enum Item{
    Decl(Decl),//处理声明单元
    FuncDef(FuncDef),//处理函数单元
}

