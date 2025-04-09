use koopa::ir::entities::{BasicBlock, Function, Value, ValueData}; // Koopa IR builder
use koopa::ir::{Program, Type, TypeKind}; // All the symbol defined in the AST
use std::collections::HashMap;

//代码生成上下文管理

// IRBuilderCtx
//需要一个结构体来保存当前IR构建器的状态，Ctx即为Context表示当前的执行上下文（运行时）
//包括当前函数、当前基本块、当前值、当前类型等
#[derive(Debug)]
pub struct IRBuilderCtx {
    // 基本块管理
    pub current_block: Option<BasicBlock>,  // 当前正在生成代码的基本块 ✔
    pub basic_blocks: Vec<BasicBlock>,     // 已创建的所有基本块（可选）
    pub block_counter: usize,              // 基本块唯一ID生成器 ✔

    // 符号管理
    pub symbol_table: SymbolTableStack,    // 层级式符号表（作用域栈），比如ident-(type,value) ✔
    pub temp_counter: usize,               // 临时变量计数器（%tmp1, %tmp2...）

    // 控制流管理
    pub control_flow_stack: Vec<ControlFlowScope>, // 控制流作用域栈（循环、条件等）//TODO,要不要把break和continue合并到这里面
    pub break_targets: Vec<BasicBlock>,    // break目标块栈（或合并到control_flow_stack）✔
    pub continue_targets: Vec<BasicBlock>, // continue目标块栈✔

    // 函数上下文
    pub current_function: Option<Function>, // 当前正在生成的函数✔
    pub function_table: HashMap<String, Function>, // 函数名到IR函数的映射✔

    // 类型系统
    //type_registry: TypeRegistry,       // 类型信息注册表，这里不需要，用koopair的Type库就可以
}

// 控制流作用域
#[derive(Debug)]
pub enum ControlFlowScope {
    Loop {
        continue_target: BasicBlock,
        break_target: BasicBlock,
    },
    If {
        break_target: BasicBlock,
    },
}

impl IRBuilderCtx{
    // 创建一个新的IRBuilderCtx实例
    pub fn new() -> Self {
        IRBuilderCtx {
            current_block: None,
            basic_blocks: Vec::new(),
            block_counter: 0,
            symbol_table: SymbolTableStack::new(),
            temp_counter: 0,
            control_flow_stack: Vec::new(),
            break_targets: Vec::new(),
            continue_targets: Vec::new(),
            current_function: None,
            function_table: HashMap::new(),
        }
    }

    // 检查是否已存在相同名称的全局符号
    pub fn check_duplicate_global_symbol(&self, name: &str) -> bool {
        // 检查全局作用域中是否存在相同名称的符号
        if let Some(global_scope) = self.symbol_table.global_scope() {
            if global_scope.contains_key(name) {
                return true;
            }
        }
        
        // 检查函数表中是否存在相同名称的函数
        self.function_table.contains_key(name)
    }
}

//错误定义
#[derive(Debug)]
pub enum SymbolError {
    NoActiveScope,
    DuplicateSymbol(String),
    SymbolNotFound(String),
}

//符号表管理
#[derive(Debug)]
pub struct SymbolTableStack{
    // 符号表栈
    /*
    栈式结构：Vec 表示嵌套的作用域层级
    - 压栈：进入新作用域（如函数、代码块）时添加新的 HashMap
    - 弹栈：退出作用域时移除最顶层的 HashMap

    哈希表存储：每个 HashMap 对应一个作用域的符号表
    - 键：符号名称（字符串）
    - 值：符号详细信息（SymbolTableEntry） 
    */
    symbol_table: Vec<HashMap<String, SymbolTableEntry>>, 
}
impl SymbolTableStack{
    // 创建一个新的符号表栈
    pub fn new() -> Self {
        SymbolTableStack {
            symbol_table: vec![HashMap::new()], // 初始化一个全局作用域
        }
    }

    // 进入新作用域（如函数、代码块）
    pub fn enter_scope(&mut self) {
        self.symbol_table.push(HashMap::new());
    }

    // 退出当前作用域
    pub fn exit_scope(&mut self) {
        self.symbol_table.pop();
    }

    // 获取当前作用域的符号表
    pub fn current_scope(&self) -> Option<&HashMap<String, SymbolTableEntry>> {
        self.symbol_table.last()
    }

    // 获取全局作用域的符号表
    pub fn global_scope(&self) -> Option<&HashMap<String, SymbolTableEntry>> {
        self.symbol_table.first()
    }

    //符号查找，从当前作用域（栈顶）向全局作用域（栈底）逐层查找
    pub fn get_symbol(&self, name:&str)->Option<&SymbolTableEntry>{
        for table in self.symbol_table.iter().rev(){
            if let Some(symbol)=table.get(name){
                return Some(symbol);
            }
        }
        return None;
    }

    //向当前作用域添加新符号
    pub fn insert_symbol(&mut self, name:&str, entry:SymbolTableEntry)->Result<(),SymbolError>{
        match self.symbol_table.last_mut() {//last_mut()返回栈顶元素(即Vec最后一个元素)的可变引用
            Some(table) => {
                table.insert(name.to_string(), entry);
                Ok(())
            }
            None => Err(SymbolError::NoActiveScope),
        }
    }

    //添加新符号表（进入新作用域）
    pub fn add_table(&mut self){
        self.symbol_table.push(HashMap::new());
    }

    //删除符号表（退出当前作用域）
    pub fn delete_table(&mut self){
        self.symbol_table.pop();
    }
}

#[derive(Debug, Clone)]
pub enum SymbolTableEntry {
    Variable(TypeKind, Value), // 变量,注意这里的Value是koopa的Value类型
    Constant(TypeKind, i32), // 常量
    Function(Function), // 函数
}

impl std::fmt::Debug for SymbolTableEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolTableEntry::Variable(ty, val) => {
                write!(f, "Variable({:?}, {:?})", ty, val)
            }
            SymbolTableEntry::Constant(ty, val) => {
                write!(f, "Constant({:?}, {:?})", ty, val)
            }
            SymbolTableEntry::Function(func) => {
                write!(f, "Function({:?})", func)
            }
        }
    }
}