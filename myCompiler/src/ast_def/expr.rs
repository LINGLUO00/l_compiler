
// # 一：精准建模语法规则
// 每个枚举类型对应一个语法规则层级：
// - `LOrExp` 处理 `||`（逻辑或）
// - `LAndExp` 处理 `&&`（逻辑与）
// - `EqExp` 处理 `==`/`!=`（相等比较）
// - `RelExp` 处理 `<`/`>`/`<=`/`>=`（关系比较）
// - `AddExp` 处理 `+`/`-`（加减）
// - `MulExp` 处理 `*`/`/`/`%`（乘除模）
// - `UnaryExp` 处理 `+`/`-`/`!`（一元运算符）
// - `PrimaryExp` 处理字面量、变量、括号表达式。

// ---

// # 二、优先级表格
// 以下是运算符优先级从高到低的完整表格：

// | 优先级 | 运算符                  | 对应的枚举类型   | 示例          |
// |--------|-------------------------|------------------|---------------|
// | 1      | `()`（括号）            | `PrimaryExp`     | `(a + b)`     |
// | 2      | `+x`/`-x`/`!x`（一元）  | `UnaryExp`       | `-5`, `!flag` |
// | 3      | `*`/`/`/`%`            | `MulExp`         | `a * b`       |
// | 4      | `+`/`-`（加减）         | `AddExp`         | `a + b`       |
// | 5      | `<`/`>`/`<=`/`>=`      | `RelExp`         | `a > 10`      |
// | 6      | `==`/`!=`               | `EqExp`          | `a == b`      |
// | 7      | `&&`（逻辑与）          | `LAndExp`        | `a && b`      |
// | 8      | `||`（逻辑或）          | `LOrExp`         | `a || b`      |

// ---

// # 三、结合性规则
// | 运算符类型       | 结合性   | 示例解析                     |
// |------------------|----------|------------------------------|
// | 一元运算符       | 右结合   | `!!x` → `!(!x)`              |
// | 二元算术运算符   | 左结合   | `a + b + c` → `(a + b) + c`  |
// | 关系/相等运算符  | 无结合性 | `a < b < c` 非法（需括号）   |
// | 逻辑运算符       | 左结合   | `a && b && c` → `(a && b) && c` |




use super::symb::*;

#[derive(Debug)]
pub enum Expr{
    LogicOrExpr(LogicOrExpr), // 一元逻辑或
}

#[derive(Debug)]
pub enum LogicOrExpr{
    LogicAndExpr(LogicAndExpr), // 一元逻辑与
    BinaryLogicOrExpr(Box<LogicOrExpr>, LogicAndExpr), // 二元逻辑或,为什么右侧不需要Box，因为LogicAndExpr是独立类型，它的大小由自身定义。左侧则需要Box，因为它是递归定义的，大小不确定
}

#[derive(Debug)]
pub enum LogicAndExpr{
    EqExpr(EqExpr), // 一元相等与不等
    BinaryLogicAndExpr(Box<LogicAndExpr>, EqExpr), // 二元逻辑与
}

#[derive(Debug)]
pub enum EqExpr{
    RelExpr(RelExpr), // 一元关系表达式,relational expression
    BinaryEqExpr(Box<EqExpr>, RelExpr), // 二元相等
    BinaryUneqExpr(Box<EqExpr>, RelExpr), // 二元不等
}

#[derive(Debug)]
pub enum RelExpr{
    AddExpr(AddExpr), // 一元加减
    BinaryLtExpr(Box<RelExpr>, AddExpr), // 二元关系
    BinaryGtExpr(Box<RelExpr>, AddExpr), // 二元关系
    BinaryLeExpr(Box<RelExpr>, AddExpr), // 二元关系
    BinaryGeExpr(Box<RelExpr>, AddExpr), // 二元关系
}

#[derive(Debug)]
pub enum AddExpr{
    MulExpr(MulExpr), // 一元乘除模
    BinaryAddExpr(Box<AddExpr>, MulExpr), // 二元加减
    BinarySubExpr(Box<AddExpr>, MulExpr), // 二元加减
}

#[derive(Debug)]
pub enum MulExpr{
    UnaryExpr(UnaryExpr), // 一元运算符,unary的音标为/ˈjuːnəri/
    BinaryMulExpr(Box<MulExpr>, UnaryExpr), // 二元乘除模
    BinaryDivExpr(Box<MulExpr>, UnaryExpr), // 二元乘除模
    BinaryModExpr(Box<MulExpr>, UnaryExpr), // 二元乘除模
}


#[derive(Debug)]
pub enum UnaryExpr{
    PrimaryExpr(PrimaryExpr), // 一元表达式，指的是字面量（如5）、变量（如a）、括号表达式（如(a+b),括号提高优先级，不参与运算）
    UnaryPlusExpr(Box<UnaryExpr>), // 一元加，即正数
    UnaryMinusExpr(Box<UnaryExpr>), // 一元减，即负数
    UnaryNotExpr(Box<UnaryExpr>), // 一元!
    FuncCall(Ident, Vec<Expr>), // 函数调用
}

#[derive(Debug)]
pub enum PrimaryExpr{
    Literal(Literal), // 字面量
    LeftVal(LeftVal), // 变量，即左值
    BracedExpr(Box<Expr>), // 括号表达式
    //注：为什么没有RightVal？因为右值已经隐含在之前的定义中
    //比如
    //字面量（如 42 → literal::INTCONST(42)）。
    // 表达式计算结果（如 c=a+b中，a + b → AddExp 节点）。
    // 函数返回值（如 c=func()，func() → UnaryExp::FuncCall）。
}

#[derive(Debug)]
pub enum LeftVal{
    // 普通变量：Vec<Exp> 为空（如 x → vec![]）。
    // 数组访问：Vec<Exp> 存储索引表达式（如 arr[2*i] → vec![2*i]）。
    // 多维数组：通过 Vec<Exp> 的长度支持任意维度（如 matrix[3][5] → vec![3, 5]）
    NormalLeftVal(Ident,Vec<Expr>),
}

#[derive(Debug)]
pub enum Literal{
    IntConst(i32), // 整数常量,因为TypeKind这个库的整数常量，只有i32,没有i64
}