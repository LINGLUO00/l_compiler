use super::symb::*;
use super::stmt::*;
use super::expr::*;

//该模块定义了以下单元的AST结构
//1.FuncDef:例如int main(int a){return 0;}。注：形式参数为FormalParameter，也叫parameter,实际参数为actualParameter，也叫argument
#[derive(Debug)]
pub enum FuncDef{
    //使用枚举是因为以后可以添加异步函数，泛型函数等
    NormalFunc(BaseType, Ident, Vec<FuncFormalParam>, Block)
}

#[derive(Debug)]
pub enum FuncFormalParam{
    //需要解决两种可能，分别是int a;和int a = 0;
    //前者为BaseType+Ident,后者为expration,即表达式
    NormalFuncFormalParam(BaseType, Ident, Option<Vec<Expr>>),//当没有初始化的时候，option是None
}

//2.Block：包含声明和语句
#[derive(Debug)]
pub enum Block{
    NormalBlock(Vec<BlockItem>),
}

#[derive(Debug)]
pub enum BlockItem{
    Decl(Decl),
    Stmt(Stmt),
}

//3.Decl:声明单元，包含变量（常量，变量）声明和函数声明
#[derive(Debug)]
pub enum Decl{
    //变量声明
    VarDecl(VarDecl),
    //常量声明
    ConstDecl(ConstDecl),
    //函数声明
    FuncDecl(BaseType, Ident, Vec<FuncFormalParam>),
}

#[derive(Debug)]
pub enum InitVal{
    //初始化值
    Expr(Expr),//比如let a = 1;
    //初始化数组
    Aggregate(Vec<Box<InitVal>>), //比如let nested_array = [[1, 2], [3, 4]]; 
}

//对于常量可能为表达式，也可能为数组
//const int a = 1;
//const int a[2]={1,1};
#[derive(Debug)]
pub enum ConstDecl {
    //比如const int a = 1;
    NormalConstDecl(BaseType, Vec<ConstDef>),//常量声明
}

#[derive(Debug)]
pub enum ConstDef{
    //常量定义,不需要BaseType，因为int a=10,b=9;中的b不需要类型
    //Vec<Expr>表示维度，int b[2]={x,x}
    NormalConstDef(Ident, Vec<Expr>, InitVal),//常量定义
}

#[derive(Debug)]
pub enum VarDecl{
    NormalVarDecl(BaseType,Vec<VarDef>)
}

#[derive(Debug)]
pub enum VarDef{
    //int a=4,b[2][3],d[2][3]={1,2,3,4,5,6};
    NormalVarDef(Ident, Vec<Expr>, Option<InitVal>),
}





