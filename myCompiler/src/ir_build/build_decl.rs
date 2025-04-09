use core::panic;

use crate::ast_def::decl::*;
use koopa::ir::{builder_traits::*, entities::ValueData, values, values::Aggregate, FunctionData, Program, Type, Value};

use super::{
    IRBuildable, IRBuilderCtx, 
    build_expr::IRExprBuildResult, context::*, 
    error::IRBuildResult, 
    build_stmt::*,
    create_local_value, emit_instructions, 
    fetch_value_metadata, params_to_koopa_ir_style, 
    params_type_to_koopa_ir_style, parse_array_dimensions, parse_nested_array_type
};

impl IRBuildable for FuncDef{
    type Output = IRBuildResult;
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        //获取函数信息
        let FuncDef::NormalFunc(ret_type,func_id ,vec_params ,block ) = self;
        let ret_ty = Type::get(ret_type.base_type);
        //处理函数的形参列表，将它们转换为Koopa IR所需的参数格式
        let mut koopa_style_params = params_to_koopa_ir_style(vec_params,program,ctx);
        //创建函数
        let func_data = FunctionData::with_param_names(format!("@{}",func_id), koopa_style_params, ret_ty.clone());
        let func = program.new_func(func_data);
        //把函数名添加到符号表,先检查函数名是否已经存在
        if ctx.function_table.contains_key(&func_id.ident) {
            return Err(format!("Function {} already exists", func_id));
        }
        ctx.function_table.insert(func_id.ident.clone(), func);

        //创建BasicBlock
        let func_data= program.func_mut(func);
        let new_block = func_data.dfg_mut().new_bb().basic_block(None);
        func_data.layout_mut().bbs_mut().extend([new_block]);//extend的方法签名为IntoIterator<Item = BasicBlock>>(&mut self, iter: T)，[BasicBlock]实现了IntoIterator，所以可以直接传入
        ctx.current_block= Some(new_block);
        ctx.current_function= Some(func);
        ctx.symbol_table.add_table();
        //处理函数的形式参数（形参）并实现参数传递机制
        for idx in 0..program.func(func).params().len() {
            let FuncFormalParam::NormalFuncFormalParam(param_type,param_ident ,dim_exprs )= &vec_params[idx];
            //为形参分配存储空间（创建局部变量）
            let formal_param_type= params_type_to_koopa_ir_style(&vec_params[idx], program, ctx);
            let formal_param=create_local_value(program, ctx).alloc(formal_param_type);
            program.func_mut(func).dfg_mut().set_value_name(formal_param, Some(format!("@{}",param_ident.ident)));
            //把形参添加到符号表
            ctx.symbol_table.insert_symbol(param_ident.ident.as_str(),SymbolTableEntry::Variable(param_type.base_type.clone(), formal_param))
                .map_err(|e| format!("Failed to insert symbol: {:?}", e))?;
            //把实参赋值给形参
            let real_param = program.func(func).params()[idx];//表示访问第几个参数，因为params()返回的是一个slice
            let assign_inst = create_local_value(program, ctx).store(real_param, formal_param);
            emit_instructions(program, ctx, [formal_param,assign_inst]);
        }
        //处理函数体
        block.ir_buildable(program,ctx)?;
        //处理返回指令
        let return_inst = create_local_value(program, ctx).ret(None);
        emit_instructions(program, ctx, [return_inst]);

        Ok(IRBuildResult::OK)
    }
}

//

impl IRBuildable for Block{
    type Output = ();
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        //获取当前基本块
        let Block::NormalBlock(stmts) = self;
        //处理当前块中的语句
        for stmt in stmts.iter() {
            stmt.ir_buildable(program, ctx)?;
        }
        //处理完语句后，删除table
        ctx.symbol_table.delete_table();
        Ok(())
    }
}

impl IRBuildable for BlockItem{
    type Output = IRBuildResult;
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        match self {
            BlockItem::Decl(decl) => decl.ir_buildable(program, ctx),
            BlockItem::Stmt(stmt) => stmt.ir_buildable(program, ctx),
        }
    }
}

impl IRBuildable for Decl{
    type Output = IRBuildResult;
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        match self {
            Decl::VarDecl(var_decl) => var_decl.ir_buildable(program, ctx),
            Decl::ConstDecl(const_decl) => const_decl.ir_buildable(program, ctx),
            Decl::FuncDecl(base_type, func_id, vec_params) => {
                //函数声明不需要处理
                Ok(IRBuildResult::OK)
            }
        }
    }
}
//接下来对InitVal进行处理

#[derive(Debug)]
pub enum IRInitValBuildResult{
    Const(i32),
    Variable(Value),
    Aggregate(Value),
}

fn create_aggregate(
    dims: &[usize],//维度
    init_vals:&[Box<InitVal>],//右值
    is_global: bool,
    program: &mut Program,
    ctx: &mut IRBuilderCtx,
) -> Result<(Value,usize), String> {
    let mut init_values_vec = Vec::new();
    let mut curr_init_val_idx = 0;
    //对于一维数组
    if dims.len() == 1 {
        for _ in 0..dims[0] {
            //确保当前的初始化值索引不越界
            let curr_init_val = if curr_init_val_idx < init_vals.len() {
                Some(&init_vals[curr_init_val_idx])
            } else {
                None
            };
            let result = match curr_init_val {
                Some(boxed_init_val)=> match boxed_init_val.as_ref() {
                    InitVal::Expr(expr) => expr.ir_buildable(program, ctx)?,
                    InitVal::Aggregate(init_vals) =>  return Err(format!("Nested array initialization is not supported")),
                    },
                None => IRExprBuildResult::Const(0), //为没有初始化的元素填充默认值
            };
            curr_init_val_idx += 1;
            let value = match result {
                IRExprBuildResult::Const(int) => {
                    if is_global{
                        program.new_value().integer(int)
                    }else{
                        create_local_value(program, ctx).integer(int)
                    }
                }
                IRExprBuildResult::Value(val) => {
                    return Err(format!("Invalid initialization value"));
                }
            };
            init_values_vec.push(value);
        }
    } else {
        //对于多维数组
        let mut elems = Vec::new();
        for _ in 0..dims[0] {
            //确保当前的初始化值索引不越界
            let curr_init_val = if curr_init_val_idx < init_vals.len() {
                Some(&init_vals[curr_init_val_idx])
            } else {
                return Err(format!("Array initialization value is not enough"));
            };
            match curr_init_val {
                Some(boxed_curr_init_val)=>match boxed_curr_init_val.as_ref(){
                    InitVal::Expr(_) => {},
                    InitVal::Aggregate(nested_init_vals) => {
                        // 递归处理嵌套数组
                        let (result, used_idx) = create_aggregate(&dims[1..], &nested_init_vals, is_global, program, ctx)?;
                        elems.push(result);
                        curr_init_val_idx += 1;
                        continue;
                    }
                }
                None => {}
            };
            //递归创建嵌套数组
            let (result, used_idx) = create_aggregate(&dims[1..], &init_vals[curr_init_val_idx..], is_global, program, ctx)?;
            //更新当前的初始化值索引
            curr_init_val_idx += used_idx;
            //将结果添加到初始化值向量中
            init_values_vec.push(result);
        }
    }
    //
    if is_global {
        //如果是全局变量，则创建一个全局变量
        Ok((
            program.new_value().aggregate(init_values_vec),
            curr_init_val_idx,
        ))
    } else {
        //如果是局部变量，则创建一个局部变量
        Ok((
            create_local_value(program, ctx).aggregate(init_values_vec),
            curr_init_val_idx,
        ))
    }
}


//将聚合类型（如数组或结构体）的初始化值转换为一系列内存存储指令，确保在程序运行时，聚合数据的每个元素被正确写入对应的内存地址
fn aggregate_to_store_insts(
    aggr: Value,
    aggr_ptr: Value,
    program: &mut Program,
    ctx: &mut IRBuilderCtx,
)->Result<(),String>{
    //如果aggr_ptr是全局的，则不需要存储指令，因为全局变量在编译时已经分配了内存
    assert!(!aggr_ptr.is_global(),"aggr_ptr should be a local variable");
    let val_data = fetch_value_metadata(aggr, program, ctx);
    let value_data = match val_data{
        //如果为Ok就提取ValueData
        Ok(value_data)=> value_data,
        //如果为Err就返回错误
        Err(err) => return Err(format!("Failed to fetch value metadata for {:?}: {}", aggr, err)),
    };
    
    match value_data.kind() {
        koopa::ir::ValueKind::Aggregate => {
            //如果是聚合类型，则需要遍历每个元素
            let aggregate_value = value_data.as_aggregate().unwrap();
            for (i, elem) in aggregate_value.elems().iter().enumerate() {
                let index = create_local_value(program, ctx).integer(i as i32);
                let elem_ptr = create_local_value(program, ctx).get_elem_ptr(aggr_ptr, index);
                emit_instructions(program, ctx, [elem_ptr]);
                
                let elem_data = fetch_value_metadata(*elem, program, ctx)?;
                if let koopa::ir::ValueKind::Aggregate = elem_data.kind() {
                    // 递归处理嵌套聚合类型
                    aggregate_to_store_insts(*elem, elem_ptr, program, ctx)?;
                } else {
                    // 对于简单类型，直接存储
                    let store_inst = create_local_value(program, ctx).store(*elem, elem_ptr);
                    emit_instructions(program, ctx, [store_inst]);
                }
            }
        }
        _ => return Err(format!("Expected aggregate type")),
    }

    Ok(())
}

// 修改 InitVal 的 IRBuildable 实现
impl IRBuildable for InitVal {
    type Output = IRInitValBuildResult;
    
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        self.build_with_dim(&[], false, program, ctx)
    }
}

// 为 InitVal 添加一个新方法
impl InitVal {
    pub fn build_with_dim(
        &self,
        dim: &[usize],
        is_global: bool,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<IRInitValBuildResult, String> {
        match self {
            InitVal::Expr(expr) => match expr.ir_buildable(program, ctx)? {
                IRExprBuildResult::Const(int) => Ok(IRInitValBuildResult::Const(int)),
                IRExprBuildResult::Value(value) => {
                    if is_global {
                        Err(format!(
                            "Non-constant expression in global variable initialization"
                        ))
                    } else {
                        Ok(IRInitValBuildResult::Variable(value))
                    }
                }
            },
            InitVal::Aggregate(init_vals) => {
                let (value, _) =
                    create_aggregate(dim, init_vals, is_global, program, ctx)?;
                Ok(IRInitValBuildResult::Aggregate(value))
            }
        }
    }
}

impl IRBuildable for ConstDecl{
    type Output = IRBuildResult;
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        //获取常量声明,我们的定义和声明是复用的
        let ConstDecl::NormalConstDecl(the_const_type, const_defs) = self;
        let const_type = &the_const_type.base_type;
        //处理常量定义
        for const_def in const_defs {
            let ConstDef::NormalConstDef(const_ident, dim_exprs, init_val) = const_def;
            let dim = parse_array_dimensions(dim_exprs, program, ctx)?.clone();
            let result = init_val.build_with_dim(&dim, ctx.current_function.is_none(), program, ctx)?;
            //在符号表中创建入口
            match result {
                IRInitValBuildResult::Const(int)=>{
                    ctx.symbol_table.insert_symbol(
                        const_ident.ident.as_str(),//存储常量的标识符
                        SymbolTableEntry::Constant(const_type.clone(), int),//存储常量的类型和值
                    ).map_err(|e| format!("Failed to insert symbol: {:?}", e))?;
                }
                IRInitValBuildResult::Variable(_)=>{
                    //不存在const int a = b;这种情况
                    return Err(format!("Constant {} must be initialized with a constant expression", const_ident));
                }
                IRInitValBuildResult::Aggregate(aggr) => {
                    //对于数组的思路：拿到数组元数据，即类型，名字等，分配空间，拿到指针列表，塞入指令集
                    let ptr_array = if ctx.current_function.is_some(){//如果是局部变量，即当前函数存在
                        let aggr_valuedata = fetch_value_metadata(aggr, program, ctx)?;
                        let addr = create_local_value(program, ctx).alloc(aggr_valuedata.ty().clone());
                        emit_instructions(program, ctx, [addr]);
                        aggregate_to_store_insts(aggr, addr, program, ctx)?;//把每一个值的指针都存入指令集
                        addr
                    }else{//如果是全局变量，即当前函数不存在，声明的时候不属于任何一个函数
                        let addr = program.new_value().global_alloc(aggr);
                        program.set_value_name(addr, Some(format!("@{}", const_ident.ident)));
                        addr
                    };
                    ctx.symbol_table.insert_symbol(
                        const_ident.ident.as_str(), 
                        SymbolTableEntry::Variable(const_type.clone(), ptr_array)
                    ).map_err(|e| format!("Failed to insert symbol: {:?}", e))?;
                }
            }
        }
        Ok(IRBuildResult::OK)
    }
}


impl IRBuildable for VarDecl{
    type Output = IRBuildResult;
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        //获取变量声明
        let VarDecl::NormalVarDecl(values_type, var_defs) = self;
        //处理变量定义
        for var_def in var_defs {
            let VarDef::NormalVarDef(var_ident, dim_exprs, init_val) = var_def;
            let dim = parse_array_dimensions(dim_exprs, program, ctx)?.clone();
            //检查维度是否存在，不存在就放回原来类型，存在就返回数组类型
            let var_type = match dim.is_empty(){
                true => values_type.base_type.clone(),
                false => parse_nested_array_type(values_type, &dim),
            };

            //为变量分配存储空间，存入符号表的时候，把变量变为koopa_ir的类型
            let var_addrs = match ctx.current_function {
                Some(func) => {
                    let var_addr = create_local_value(program, ctx).alloc(Type::get(var_type.clone()));
                    program.func_mut(func).dfg_mut().set_value_name(var_addr, Some(format!("@{}", var_ident.ident)));
                    emit_instructions(program, ctx, [var_addr]);
                    
                    //如果有初始值
                    if let Some(init_val) = init_val {
                        let init_val_result = init_val.build_with_dim(&dim, false, program, ctx)?;
                        match init_val_result {
                            IRInitValBuildResult::Const(int) => {
                                let int_val = create_local_value(program, ctx).integer(int);
                                let store_inst = create_local_value(program, ctx).store(int_val, var_addr);
                                emit_instructions(program, ctx, [store_inst]);
                            },
                            IRInitValBuildResult::Variable(value) => {
                                let store_inst = create_local_value(program, ctx).store(value, var_addr);
                                emit_instructions(program, ctx, [store_inst]);
                            },
                            IRInitValBuildResult::Aggregate(aggr) => {
                                aggregate_to_store_insts(aggr, var_addr, program, ctx)?;
                            }
                        }
                    }
                    var_addr
                }
                //如果是全局的
                None => {
                    //检查有没有重名
                    if ctx.function_table.contains_key(&var_ident.ident) {
                        return Err(format!("Variable {} already exists", var_ident));
                    }
                    //分配空间
                    let var_addr = if let Some(init_val) = init_val {
                        match init_val.build_with_dim(&dim, true, program, ctx)? {
                            IRInitValBuildResult::Const(int) => {
                                program.new_value().global_alloc(program.new_value().integer(int))
                            }
                            IRInitValBuildResult::Variable(_) => {
                                return Err(format!("Global variable must be initialized with a constant expression"));
                            }
                            IRInitValBuildResult::Aggregate(aggr) => {
                                program.new_value().global_alloc(aggr)
                            }
                        }
                    } else {
                        let zero_init = program.new_value().zero_init(Type::get(var_type.clone()));
                        program.new_value().global_alloc(zero_init)
                    };
                    program.set_value_name(var_addr, Some(format!("@{}", var_ident.ident)));
                    var_addr
                }
            };
            //将变量添加到符号表
            ctx.symbol_table.insert_symbol(
                var_ident.ident.as_str(),
                SymbolTableEntry::Variable(var_type.clone(), var_addrs),
            ).map_err(|e| format!("Failed to insert symbol: {:?}", e))?;
        }
        Ok(IRBuildResult::OK)
    }
}

