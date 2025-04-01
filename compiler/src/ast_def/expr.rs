#[derive(Debug)]
pub enum Expr {
    Literal(Literal),
}

#[derive(Debug)]
pub enum Literal {
    Int(i32),
}
