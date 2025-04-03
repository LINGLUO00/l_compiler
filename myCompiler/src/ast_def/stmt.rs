
// | 变体               | 解析内容                     | 实际代码示例                      |
// |--------------------|----------------------------|---------------------------------|
// | `AssignStmt`       | 赋值语句                     | `x = 10;`                       |
// | `Exp`              | 表达式或空语句                | `print("Hi");` 或 `;`           |
// | `Block`            | 代码块                      | `{ let x = 1; }`                |
// | `IfStmt`           | 条件分支（if-else）          | `if (x>0) { ... } else { ... }` |
// | `WhileStmt`        | while 循环                 | `while (true) { ... }`          |
// | `BreakStmt`        | 跳出循环                    | `break;`                        |
// | `ContinueStmt`     | 继续下一轮循环               | `continue;`                     |
// | `ReturnStmt`       | 函数返回值或结束              | `return 42;` 或 `return;`       |




/*
1. **Stmt（语句）**：
   - 可以是`UnmatchedStmt`或`MatchedStmt`，通过枚举`Stmt::UnmatchedStmt`和`Stmt::MatchedStmt`区分。

2. **UnmatchedStmt（非匹配语句）**：
   - 包含不完整或可能引发歧义的结构：
     - `if`语句**不带`else`**（无论`then`分支是`Matched`还是`Unmatched`）。
     - `if`语句**带`else`，但`else`分支是`Unmatched`**。
     - `while`循环体为`Unmatched`的语句。
   - 这些结构可能无法明确闭合（如没有`else`的`if`），导致后续`else`归属不清。

3. **MatchedStmt（匹配语句）**：
   - 包含完整且无歧义的结构：
     - 赋值、表达式、块语句。
     - **带`else`的`if`**（且`then`和`else`均为`Matched`）。
     - `while`循环体为`Matched`的语句。
     - 控制语句（`break`/`continue`/`return`）。
   - 这些结构闭合明确，不会导致悬挂`else`。
*/


use super::decl::*;
use super::expr::*;

#[derive(Debug)]
pub enum Stmt{
    //这两个字段的目的是让类似于多个if，else嵌套的时候，让else强制与为被匹配的if匹配
    UnmatchedStmt(UnmatchedStmt),
    MatchedStmt(MatchedStmt),
}

#[derive(Debug)]
pub struct UnmatchedStmt{
    pub NormalUnmatchedStmt:BasicStmt,
}

#[derive(Debug)]
pub struct MatchedStmt {
    pub NormalMatchedStmt: BasicStmt,
}


#[derive(Debug)]
pub enum BasicStmt {
    AssignStmt(LeftVal, Expr),
    Expr(Option<Expr>),
    Block(Block),
    IfStmt(Expr, Box<BasicStmt>, Box<Option<BasicStmt>>),
    WhileStmt(Expr, Box<BasicStmt>),
    BreakStmt,
    ContinueStmt,
    ReturnStmt(Option<Expr>),
}
