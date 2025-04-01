use crate::ast_def::expr::Expr;
#[derive(Debug)]
pub enum Stmt{
    //目前只有return 0；这条语句
    MatchedStmt(MatchedStmt),
}

#[derive(Debug)]
pub struct MatchedStmt{
    pub default:BasicStmt,
}

#[derive(Debug)]
pub enum BasicStmt{
    ReturnStmt(Expr),
}
//pub ret_num:i32,
