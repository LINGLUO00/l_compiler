mod build_decl;
mod build_expr;
mod build_stmt;
mod context;
mod error;

use build_expr::IRExprBuildResult;

use crate::ast_def::*;
use crate::ast_def::decl::*;
use crate::ast_def::expr::*;
use crate::ast_def::symb::*;

use context::IRBuilderCtx;
use error::IRBuildResult;
use koopa::ir::builder_traits::*;
use koopa::ir::entities::{BasicBlock,Value, ValueData}; // Koopa IR builder
use koopa::ir::{Program, Type, TypeKind}; // All the symbol defined in the AST
use std::collections::HashMap;
use core::result::Result;


//构建IR的主函数，只需要让该函数构建comp_unit，其他的由comp_unit递归构建
pub fn build_ir (comp_unit:&CompUnit)->Result<Program,String>{
    let mut program = Program::new();
    let mut ctx = IRBuilderCtx::new();
    comp_unit.ir_buildable(&mut program, &mut ctx)?;
    Ok(program)
}


pub trait IRBuildable{//able后缀，表示实现这个trait就可以被构建的意思，是类似于interface的命名
    type Output;//type Output 是一个关联类型（Associated Type），它定义了该 trait 的实现者需要指定的一个具体类型。
    //比如对于func，type Output = Function; 即，具体类型由实现者指定
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String>;
}

impl IRBuildable for CompUnit {
    type Output = IRBuildResult;
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        //利用Type提供的库函数，先把这些库函数注册到符号表
        let lib_func = vec![
            ("getint",vec![], Type::get_i32()),//第二个参数是函数参数列表
            ("getch",vec![], Type::get_i32()),
            ("getarray",vec![Type::get_pointer(Type::get_i32())], Type::get_i32()),
            ("putint", vec![Type::get_i32()], Type::get_unit()),
            ("putch", vec![Type::get_i32()], Type::get_unit()),
            (
                "putarray",
                vec![Type::get_i32(), Type::get_pointer(Type::get_i32())],
                Type::get_unit(),
            ),
            ("starttime", vec![], Type::get_unit()),
            ("stoptime", vec![], Type::get_unit()),
        ];
        for(name, params_ty,ret_ty) in lib_func{
            let func_data = koopa::ir::FunctionData::new_decl(format!("@{}",name), params_ty, ret_ty);//这里是声明，不需要函数参数标识符
            let func = program.new_func(func_data);
            ctx.function_table.insert(name.to_string(), func);
        }

        //遍历执行各个编译单元
        let CompUnit{items }= self;
        for unit_item in items{
            unit_item.ir_buildable(program, ctx)?;
        }
        Ok(IRBuildResult::OK)
    }
}

impl IRBuildable for Item {
    type Output = IRBuildResult;
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        match self {
            Item::Decl(decl) => {
                decl.ir_buildable(program, ctx)?;
                Ok(IRBuildResult::OK)
            },
            Item::FuncDef(func_def) => {
                func_def.ir_buildable(program, ctx)?;
                Ok(IRBuildResult::OK)
            },
        }
    }
}

//以下为一些辅助方法
//获取值的元数据，其实就是获取一个Value结构体，包含该变量的类型，值等信息
fn fetch_value_metadata(value_metadata:Value, program:&mut Program, ctx:&mut IRBuilderCtx)->Result<ValueData,String>{
    //区分值来源于全局还是局部
    //如果是全局就直接clone，如果是局部就从当前作用域获取
    if value_metadata.is_global(){
        let value_data = program.borrow_value(value_metadata).clone();
        //如果value不存在，返回错误信息，这里好像没有办法添加错误处理操作
        return Ok(value_data);
    }
    else{
        let value_data = program.func(ctx.current_function.unwrap())
            .dfg()
            .value(value_metadata)
            .clone();
        return Ok(value_data);
            
    }
}

//创建新的局部值
fn create_local_value<'a>(
    program:&'a mut Program,
    ctx:&'a mut IRBuilderCtx,
)->koopa::ir::builder::LocalBuilder<'a>{
    program
        .func_mut(ctx.current_function.expect("No current function,you can create_global_value instead !!"))
        .dfg_mut()
        .new_value()
}

//分配带有自动命名的基本块(于内存中分配)
fn create_basic_block(
    program:&mut Program,
    ctx:&mut IRBuilderCtx,
    name:&str,
)->BasicBlock{
    let block = program
        .func_mut(ctx.current_function.expect("No current function, create_basic_block failed!"))
        .dfg_mut()
        .new_bb()
        .basic_block(Some(format!("%bb{}{}", ctx.block_counter, name)));
    ctx.block_counter += 1;
    block
}

//向当前基本块插入指令流,emit为指令发射的意思
fn emit_instructions<T>(
    program:&mut Program,
    ctx:&mut IRBuilderCtx,
    instructions:T,
) where
    T: IntoIterator<Item = Value>,//IntoIterator是一个标准库中的trait，用于将一个类型转换为迭代器。任何实现了IntoIterator的类型都可以被迭代。本语句中迭代器的元素类型必须是Value
{
    program
        .func_mut(ctx.current_function.expect("No current function, emit_instructions failed!"))//获取当前函数
        .layout_mut()// Layout of instructions and basic blocks in a function.
        // `Layout` maintains the order of instructions ([`Value`]) and basic blocks ([`BasicBlock`]) in function.
        .bb_mut(ctx.current_block.expect("No current block, emit_instructions failed!"))//获取当前基本块
        .insts_mut()//获取指令列表
        .extend(instructions)//目的是将指令插入到当前基本块中，如果它返回None，就会触发ok_or_else，ok_or_else就会将None转换为Err(String),并通过闭包生成一个错误信息
}


//向当前函数的数据流图追加基本块集合
fn append_basic_block<T>(
    program:&mut Program,
    ctx:&IRBuilderCtx,
    basic_blocks:T
) where
    T:IntoIterator<Item=BasicBlock>,
{
    program
        .func_mut(ctx.current_function.expect("No current func, append_basic_block failed!"))
        .layout_mut()
        .bbs_mut()
        .extend(basic_blocks);
}

//解析数组维度常量表达式(parse:解析，dimensions:维度)
//验证数组维度表达式：确保数组的每个维度长度是 编译时常量（而非运行时变量）。
//收集维度值：将合法的常量维度值转换为 usize 列表，供后续数组类型构建和内存分配使用。
fn parse_array_dimensions(
    dimensions_exprs: &Vec<Expr>,// 数组维度表达式列表（如 int a[3][5] =>[3, 5]）
    program: &mut Program,
    ctx: &mut IRBuilderCtx,
) -> Result<Vec<usize>, String> {//返回维度值列表[3,5]
    let mut dim_vec = Vec::new();
    for dim_expr in dimensions_exprs {
        match dim_expr.ir_buildable(program, ctx)? {
            IRExprBuildResult::Const(int)=>dim_vec.push(int as usize),//如果是常量表达式，就将其转换为usize
            IRExprBuildResult::Value(_)=>{
                return Err(format!("Non-constant expression found in array dimensions: {:?}", dim_expr));
            }
        }
    }
    Ok(dim_vec.clone())
}

//递归构建多维数组类型
//根据基础类型（BaseType）和维度列表（dimansions）,比如int[2][3] 将构建为 Array(Array(Int, 3), 2)
//netsted表示嵌套的意思
fn parse_nested_array_type(
    arr_type: &BaseType,      // 数组的基础类型
    dimensions: &[usize], // 数组维度列表,[usize]为连续元素的切片，无所有权，为动态大小（DST），而vec有所有权，我们这里并不需要对维度列表进行修改，用切片可以节省内存
) -> TypeKind {
    if dimensions.is_empty(){
        return arr_type.base_type.clone();
    }
    let arr_typekind = parse_nested_array_type(arr_type, &dimensions[1..]);
    TypeKind::Array(Type::get(arr_typekind), dimensions[0])
}

//对于函数参数int a[3]，我们需要将其转换为Koopa IR所需的格式
//比如int a[3] => %a_param,i32
//就是说我们需要将参数的名字前面加上_param后缀，并且如果是数组类型的话，我们需要将其转换为指针类型
//1.转换类型
fn params_type_to_koopa_ir_style(
    param:&FuncFormalParam, // 函数参数
    program: &mut Program,
    ctx: &mut IRBuilderCtx,
) -> Type {
    let FuncFormalParam::NormalFuncFormalParam(param_type, _, dim_exprs)= param;
    let fmt_param_type = match dim_exprs {
        Some(vec_dim_expr) => {
            let dim = parse_array_dimensions(vec_dim_expr, program,ctx).unwrap();
            Type::get_pointer(Type::get(parse_nested_array_type(param_type, &dim)))
        }
        None => Type::get(param_type.base_type.clone())
    };
    return fmt_param_type;
}
//2.转换参数列表
#[allow(unused_variables)]
#[warn(unreachable_patterns)]
fn params_to_koopa_ir_style(
    params: &[FuncFormalParam], // 函数参数列表
    program: &mut Program,
    ctx: &mut IRBuilderCtx,
) -> Vec<(Option<String>, Type)> {
    let mut koopa_style_params = Vec::new();
    for idx in 0..params.len() {
        match &params[idx] {
            FuncFormalParam::NormalFuncFormalParam(param_type, param_ident, dim_exprs) => {
                let fmt_param_type = params_type_to_koopa_ir_style(&params[idx], program, ctx);
                koopa_style_params.push((Some(format!("%{}_param", param_ident.ident)), fmt_param_type));
            },
            _ => {},
        }
    }
    koopa_style_params
}