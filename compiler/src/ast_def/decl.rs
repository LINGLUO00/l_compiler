pub use super::stmt::Stmt;
pub use super::symbols::*;
//函数头，如int main()
#[derive(Debug)]
// pub struct FuncDef {
//     pub func_type: FuncType,
//     pub ident: String,
//     pub block: Block,
// }
pub enum FuncDef{
    Default(FuncType, Ident, Block),
}


#[derive(Debug)]
// pub struct Block {
//     pub stmt: Stmt,
// }
pub enum Block{
    Default(Vec<BlockItem>),
}

#[derive(Debug)]
pub enum BlockItem{
    Stmt(Stmt),
    //Decl(Decl),
}


