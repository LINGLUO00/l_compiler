use crate::ast_def::expr::*;
use koopa::ir::{builder_traits::*, Program, Type, TypeKind, Value};

use super::{
    IRBuildable, IRBuilderCtx,
    context::*,
    error::IRBuildResult,
    append_basic_block, 
    create_basic_block, create_local_value, emit_instructions, 
    fetch_value_metadata, params_to_koopa_ir_style, 
    params_type_to_koopa_ir_style, parse_array_dimensions, parse_nested_array_type,
};

#[derive(Debug)]
pub enum IRExprBuildResult {
    Const(i32),
    Value(Value),
}

#[derive(Debug)]
pub enum IRLeftValBuildResult {
    Const(i32),
    TempVal(Value),
    Addr(Value),
}

impl IRBuildable for Expr{
    type Output = IRExprBuildResult;
    fn ir_buildable(
        &self,
        program:&mut Program,
        ctx:& mut IRBuilderCtx,
    )->Result<Self::Output, String>{
        match self{
            Expr::LogicOrExpr(or_exp)=>or_exp.ir_buildable(program, ctx),
        }
    }
}

//根据两个表达式的构建结果（可能是常量或 IR 值），生成对应的二元操作结果。
//如果是常量表达式，则直接计算结果；如果是非常量表达式，则生成 IR 指令并返回新的 IR 值
fn build_binary_from_build_results(
    result1: IRExprBuildResult,
    result2: IRExprBuildResult,
    program: &mut Program,
    ctx: &mut IRBuilderCtx,
    binary_op: koopa::ir::BinaryOp,
) -> Result<IRExprBuildResult, String> {
    //如果两个操作数都是常量，则直接在编译时计算结果，避免生成运行时指令。这是一种常量折叠优化，提升运行时性能
    if let (IRExprBuildResult::Const(int1), IRExprBuildResult::Const(int2)) = (&result1, &result2) {
        Ok(IRExprBuildResult::Const(match binary_op {
            koopa::ir::BinaryOp::NotEq => (int1 != int2) as i32,
            koopa::ir::BinaryOp::Eq => (int1 == int2) as i32,
            koopa::ir::BinaryOp::Gt => (int1 > int2) as i32,
            koopa::ir::BinaryOp::Lt => (int1 < int2) as i32,
            koopa::ir::BinaryOp::Ge => (int1 >= int2) as i32,
            koopa::ir::BinaryOp::Le => (int1 <= int2) as i32,
            koopa::ir::BinaryOp::Add => int1 + int2,
            koopa::ir::BinaryOp::Sub => int1 - int2,
            koopa::ir::BinaryOp::Mul => int1 * int2,
            koopa::ir::BinaryOp::Div => int1 / int2,
            koopa::ir::BinaryOp::Mod => int1 % int2,
            koopa::ir::BinaryOp::And => int1 & int2,
            koopa::ir::BinaryOp::Or => int1 | int2,
            koopa::ir::BinaryOp::Xor => int1 ^ int2,
            koopa::ir::BinaryOp::Shl => todo!(),
            koopa::ir::BinaryOp::Shr => todo!(),
            koopa::ir::BinaryOp::Sar => todo!(),
        }))
    } else {
        //处理非常量表达式的情况
        //将非常量表达式的结果转换为 IR 中的值（Value），以便后续生成指令。常量会被包装为局部值，非常量直接使用已有的值
        let value1 = match result1 {
            IRExprBuildResult::Const(int) => {
                create_local_value(program, ctx).integer(int)
            }
            IRExprBuildResult::Value(value) => value,
        };
        let value2 = match result2 {
            IRExprBuildResult::Const(int) => {
                create_local_value(program, ctx).integer(int)
            }
            IRExprBuildResult::Value(value) => value,
        };
        //根据二元操作符（binary_op）生成对应的 IR 指令，并将结果存储为新的局部值（new_value）。同时将生成的指令插入到 IR 中
        let new_value = create_local_value(program, ctx).binary(binary_op, value1, value2);
        emit_instructions(program, ctx, [new_value]);
        Ok(IRExprBuildResult::Value(new_value))
    }
}


//
impl IRBuildable for LogicOrExpr {
    type Output = IRExprBuildResult;
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        match self {
            //如果当前逻辑或表达式是一个单一的逻辑与表达式（LAndExp），直接调用其 ir_buildable 方法生成对应的 IR
            LogicOrExpr::LogicAndExpr(exp) => exp.ir_buildable(program, ctx),
            //如果当前逻辑或表达式是一个二元逻辑或表达式（BinaryLOrExp），先递归构建左操作数（exp1），然后根据其结果决定如何处理右操作数（exp2）
            LogicOrExpr::BinaryLogicOrExpr(exp1, exp2) => {

                let exp1_build_result = exp1.ir_buildable(program, ctx)?;

                match exp1_build_result {
                    // If exp1 is variable.
                    IRExprBuildResult::Value(value1) => {
                        //创建基本块：为逻辑或的短路行为创建两个基本块（block1 和 block_end）
                        let block1 = create_basic_block(program, ctx, "LogicOr_if_block");
                        let block_end = create_basic_block(program, ctx, "LogicOr_if_block_end");
                        append_basic_block(program, ctx, [block1, block_end]);
                        //创建一个局部值 result_ptr，用于存储逻辑或的结果
                        let result_ptr = create_local_value(program, ctx).alloc(Type::get_i32());
                        program.func_mut(ctx.current_function.unwrap()).dfg_mut().set_value_name(result_ptr, Some(format!("@LogicOr_result")));
                        //根据 exp1 的值生成条件分支指令。如果 exp1 为假（0），跳转到 block1 继续计算 exp2；否则直接跳转到 block_end
                        let one = create_local_value(program, ctx).integer(1);
                        let zero = create_local_value(program, ctx).integer(0);
                        let store_inst = create_local_value(program, ctx).store(one, result_ptr);

                        let should_continue = create_local_value(program, ctx).binary(
                            koopa::ir::BinaryOp::Eq,
                            value1,
                            zero,
                        );
                        let branch_inst = create_local_value(program, ctx).branch(should_continue, block1, block_end);
                        emit_instructions(program, ctx, [result_ptr, store_inst, should_continue, branch_inst]);
                        //在 block1 中构建 exp2 的值，并将其结果存储到 result_ptr 中
                        ctx.current_block = Some(block1);
                        //递归构建 exp2 的 IR
                        let result2 = build_binary_from_build_results(
                            IRExprBuildResult::Const(0),
                            exp2.ir_buildable(program, ctx)?,
                            program,
                            ctx,
                            koopa::ir::BinaryOp::NotEq,
                        )?;
                        let value2 = match result2 {
                            IRExprBuildResult::Const(i2) => {
                                create_local_value(program, ctx).integer(i2)
                            }
                            IRExprBuildResult::Value(v2) => v2,
                        };
                        let store_new_inst = create_local_value(program, ctx).store(value2, result_ptr);
                        let jmp_inst = create_local_value(program, ctx).jump(block_end);
                        emit_instructions(program, ctx, [store_new_inst, jmp_inst]);

                        ctx.current_block = Some(block_end);

                        //在 block_end 中加载逻辑或的最终结果，并返回给调用方
                        let loaded_result = create_local_value(program, ctx).load(result_ptr);
                        emit_instructions(program, ctx, [loaded_result]);
                        Ok(IRExprBuildResult::Value(loaded_result))
                    }
                    //处理 exp1 为常量的情况
                    // 如果 exp1 是常量且不为 0，直接返回逻辑或的结果为 1（短路优化）
                    //如果 exp1 为 0，递归构建 exp2 并返回其结果
                    IRExprBuildResult::Const(i1) => {
                        if i1 != 0 {
                            Ok(IRExprBuildResult::Const(1))
                        } else {
                            build_binary_from_build_results(
                                IRExprBuildResult::Const(0),
                                exp2.ir_buildable(program, ctx)?,
                                program,
                                ctx,
                                koopa::ir::BinaryOp::NotEq,
                            )
                        }
                    }
                }
            }
        }
    }
}

impl IRBuildable for LogicAndExpr {
    type Output = IRExprBuildResult;
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        match self {
            LogicAndExpr::EqExpr(exp) => exp.ir_buildable(program, ctx),
            LogicAndExpr::BinaryLogicAndExpr(exp1, exp2) => {
                
                let exp1_build_result = exp1.ir_buildable(program, ctx)?;

                match exp1_build_result {
                    //如果 exp1 是变量，则创建两个基本块（block1 和 block_end），用于处理逻辑与的短路行为
                    IRExprBuildResult::Value(value1) => {
                        let block1 = create_basic_block(program, ctx, "LogicAnd_if_block");
                        let block_end = create_basic_block(program, ctx, "LogicAnd_if_block_end");
                        append_basic_block(program, ctx, [block1, block_end]);

                        let result_ptr = create_local_value(program, ctx).alloc(Type::get_i32());
                        program.func_mut(ctx.current_function.unwrap()).dfg_mut().set_value_name(result_ptr, Some(format!("@LogicAnd_result")));
                        let zero = create_local_value(program, ctx).integer(0);
                        let store_inst = create_local_value(program, ctx).store(zero, result_ptr);

                        let should_continue = create_local_value(program, ctx).binary(
                            koopa::ir::BinaryOp::NotEq,
                            value1,
                            zero,
                        );
                        let branch_inst = create_local_value(program, ctx).branch(should_continue, block1, block_end);
                        emit_instructions(
                            program,
                            ctx,
                            [result_ptr, store_inst, should_continue, branch_inst],
                        );
                        //在 block1 中构建 exp2 的值，并将其结果存储到 result_ptr 中
                        ctx.current_block = Some(block1);
                        let result2 = build_binary_from_build_results(
                            IRExprBuildResult::Const(0),
                            exp2.ir_buildable(program, ctx)?,
                            program,
                            ctx,
                            koopa::ir::BinaryOp::NotEq,
                        )?;
                        let value2 = match result2 {
                            IRExprBuildResult::Const(i2) => {
                                create_local_value(program, ctx).integer(i2)
                            }
                            IRExprBuildResult::Value(v2) => v2,
                        };
                        let store_new_inst = create_local_value(program, ctx).store(value2, result_ptr);
                        let jmp_inst = create_local_value(program, ctx).jump(block_end);
                        emit_instructions(
                            program,
                            ctx,
                            [store_new_inst, jmp_inst],
                        );

                        ctx.current_block = Some(block_end);
                        let loaded_result = create_local_value(program, ctx).load(result_ptr);
                        emit_instructions(program, ctx, [loaded_result]);
                        Ok(IRExprBuildResult::Value(loaded_result))
                    }

                    //处理 exp1 为常量的情况
                    IRExprBuildResult::Const(i1) => {
                        if i1 == 0 {
                            Ok(IRExprBuildResult::Const(0))
                        } else {
                            build_binary_from_build_results(
                                IRExprBuildResult::Const(0),
                                exp2.ir_buildable(program, ctx)?,
                                program,
                                ctx,
                                koopa::ir::BinaryOp::NotEq,
                            )
                        }
                    }
                }
            }
        }
    }
}

impl IRBuildable for EqExpr {
    type Output = IRExprBuildResult;
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx
    ) -> Result<Self::Output, String> {
        match self {
            EqExpr::RelExpr(exp) => exp.ir_buildable(program, ctx),
            EqExpr::BinaryEqExpr(exp1, exp2) => build_binary_from_build_results(
                exp1.ir_buildable(program, ctx)?,
                exp2.ir_buildable(program, ctx)?,
                program,
                ctx,
                koopa::ir::BinaryOp::Eq,
            ),
            EqExpr::BinaryUneqExpr(exp1, exp2) => build_binary_from_build_results(
                exp1.ir_buildable(program, ctx)?,
                exp2.ir_buildable(program, ctx)?,
                program,
                ctx,
                koopa::ir::BinaryOp::NotEq,
            ),
        }
    }
}

impl IRBuildable for RelExpr {
    type Output = IRExprBuildResult;
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        match self {
            RelExpr::AddExpr(exp) => exp.ir_buildable(program, ctx),
            RelExpr::BinaryLtExpr(exp1, exp2) => build_binary_from_build_results(
                exp1.ir_buildable(program, ctx)?,
                exp2.ir_buildable(program, ctx)?,
                program,
                ctx,
                koopa::ir::BinaryOp::Lt,
            ),
            RelExpr::BinaryGtExpr(exp1, exp2) => build_binary_from_build_results(
                exp1.ir_buildable(program, ctx)?,
                exp2.ir_buildable(program, ctx)?,
                program,
                ctx,
                koopa::ir::BinaryOp::Gt,
            ),
            RelExpr::BinaryLeExpr(exp1, exp2) => build_binary_from_build_results(
                exp1.ir_buildable(program, ctx)?,
                exp2.ir_buildable(program, ctx)?,
                program,
                ctx,
                koopa::ir::BinaryOp::Le,
            ),
            RelExpr::BinaryGeExpr(exp1, exp2) => build_binary_from_build_results(
                exp1.ir_buildable(program, ctx)?,
                exp2.ir_buildable(program, ctx)?,
                program,
                ctx,
                koopa::ir::BinaryOp::Ge,
            ),
        }
    }
}

impl IRBuildable for AddExpr {
    type Output = IRExprBuildResult;
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        match self {
            AddExpr::MulExpr(exp) => exp.ir_buildable(program, ctx),
            AddExpr::BinaryAddExpr(exp1, exp2) => build_binary_from_build_results(
                exp1.ir_buildable(program, ctx)?,
                exp2.ir_buildable(program, ctx)?,
                program,
                ctx,
                koopa::ir::BinaryOp::Add,
            ),
            AddExpr::BinarySubExpr(exp1, exp2) => build_binary_from_build_results(
                exp1.ir_buildable(program, ctx)?,
                exp2.ir_buildable(program, ctx)?,
                program,
                ctx,
                koopa::ir::BinaryOp::Sub,
            ),
        }
    }
}

impl IRBuildable for MulExpr {
    type Output = IRExprBuildResult;
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        match self {
            MulExpr::UnaryExpr(exp) => exp.ir_buildable(program, ctx),
            MulExpr::BinaryMulExpr(exp1, exp2) => build_binary_from_build_results(
                exp1.ir_buildable(program, ctx)?,
                exp2.ir_buildable(program, ctx)?,
                program,
                ctx,
                koopa::ir::BinaryOp::Mul,
            ),
            MulExpr::BinaryDivExpr(exp1, exp2) => build_binary_from_build_results(
                exp1.ir_buildable(program, ctx)?,
                exp2.ir_buildable(program, ctx)?,
                program,
                ctx,
                koopa::ir::BinaryOp::Div,
            ),
            MulExpr::BinaryModExpr(exp1, exp2) => build_binary_from_build_results(
                exp1.ir_buildable(program, ctx)?,
                exp2.ir_buildable(program, ctx)?,
                program,
                ctx,
                koopa::ir::BinaryOp::Mod,
            ),
        }
    }
}

impl IRBuildable for UnaryExpr {
    type Output = IRExprBuildResult;
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        match self {
            UnaryExpr::PrimaryExpr(exp) => exp.ir_buildable(program, ctx),
            UnaryExpr::UnaryPlusExpr(exp) => exp.ir_buildable(program, ctx),
            UnaryExpr::UnaryMinusExpr(exp) => {
                let exp_result = exp.ir_buildable(program, ctx)?;
                if let IRExprBuildResult::Const(int) = exp_result {
                    Ok(IRExprBuildResult::Const(-int))
                } else {
                    build_binary_from_build_results(
                        IRExprBuildResult::Const(0),
                        exp_result,
                        program,
                        ctx,
                        koopa::ir::BinaryOp::Sub,
                    )
                }
            },
            UnaryExpr::UnaryNotExpr(exp) => {
                let exp_result = exp.ir_buildable(program, ctx)?;
                if let IRExprBuildResult::Const(int) = exp_result {
                    Ok(IRExprBuildResult::Const((int == 0) as i32))
                } else {
                    build_binary_from_build_results(
                        IRExprBuildResult::Const(0),
                        exp_result,
                        program,
                        ctx,
                        koopa::ir::BinaryOp::Eq,
                    )
                }
            },
            UnaryExpr::FuncCall(ident, args) => {
                // 查找函数并克隆函数引用，避免后续可变借用冲突
                let func_ref = match ctx.function_table.get(&ident.ident) {
                    Some(func) => *func, // 解引用并获取值的拷贝
                    None => return Err(format!("Function {} not found", ident.ident)),
                };
                
                // 处理参数
                let mut arg_values = Vec::new();
                for arg in args {
                    let arg_result = arg.ir_buildable(program, ctx)?;
                    let arg_value = match arg_result {
                        IRExprBuildResult::Const(int) => 
                            create_local_value(program, ctx).integer(int),
                        IRExprBuildResult::Value(value) => value,
                    };
                    arg_values.push(arg_value);
                }
                
                // 创建函数调用 - 使用已保存的函数引用
                let call_inst = create_local_value(program, ctx).call(func_ref, arg_values);
                emit_instructions(program, ctx, [call_inst]);
                
                Ok(IRExprBuildResult::Value(call_inst))
            },
        }
    }
}

// 实现 IRBuildable for LeftVal
impl IRBuildable for LeftVal {
    type Output = IRLeftValBuildResult;
    
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        match self {
            LeftVal::NormalLeftVal(ident, index_exprs) => {
                // 首先处理没有索引的情况
                if index_exprs.is_empty() {
                    // 获取符号并处理
                    let symbol_opt = ctx.symbol_table.get_symbol(&ident.ident);
                    
                    match symbol_opt {
                        Some(symbol) => match &symbol.data {
                            SymbolData::Variable(_, val) => Ok(IRLeftValBuildResult::Addr(*val)),
                            SymbolData::Constant(_, value) => Ok(IRLeftValBuildResult::Const(*value)),
                            SymbolData::Function(_) => Err(format!("Function {} cannot be used as a left value", ident.ident)),
                        },
                        None => Err(format!("Symbol not found: {}", ident.ident)),
                    }
                } else {
                    // 有索引的情况 - 需要先获取变量地址和所有索引，然后处理
                    
                    // 1. 先获取变量的基地址
                    let base_addr = match ctx.symbol_table.get_symbol(&ident.ident) {
                        Some(symbol) => match &symbol.data {
                            SymbolData::Variable(_, val) => *val,
                            _ => return Err(format!("Cannot index into non-array variable: {}", ident.ident)),
                        },
                        None => return Err(format!("Symbol not found: {}", ident.ident)),
                    };
                    
                    // 2. 计算所有的索引
                    let mut processed_indices = Vec::new();
                    for idx_expr in index_exprs {
                        // 处理每个索引表达式
                        let result = idx_expr.ir_buildable(program, ctx)?;
                        match result {
                            IRExprBuildResult::Const(int) => {
                                let idx_val = create_local_value(program, ctx).integer(int);
                                processed_indices.push(idx_val);
                            },
                            IRExprBuildResult::Value(value) => {
                                processed_indices.push(value);
                            },
                        }
                    }
                    
                    // 3. 生成数组元素访问指令
                    let mut curr_ptr = base_addr;
                    for idx in processed_indices {
                        let elem_ptr = create_local_value(program, ctx).get_elem_ptr(curr_ptr, idx);
                        emit_instructions(program, ctx, [elem_ptr]);
                        curr_ptr = elem_ptr;
                    }
                    
                    Ok(IRLeftValBuildResult::Addr(curr_ptr))
                }
            }
        }
    }
}

// 实现 IRBuildable for Literal
impl IRBuildable for Literal {
    type Output = IRExprBuildResult;
    
    fn ir_buildable(
        &self,
        _program: &mut Program,
        _ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        match self {
            Literal::IntConst(val) => Ok(IRExprBuildResult::Const(*val)),
        }
    }
}

// 实现 IRBuildable for PrimaryExpr
impl IRBuildable for PrimaryExpr {
    type Output = IRExprBuildResult;
    
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        match self {
            PrimaryExpr::Literal(lit) => lit.ir_buildable(program, ctx),
            PrimaryExpr::LeftVal(lval) => {
                match lval.ir_buildable(program, ctx)? {
                    IRLeftValBuildResult::Const(val) => Ok(IRExprBuildResult::Const(val)),
                    IRLeftValBuildResult::TempVal(val) => Ok(IRExprBuildResult::Value(val)),
                    IRLeftValBuildResult::Addr(addr) => {
                        // 加载地址中的值
                        let load_inst = create_local_value(program, ctx).load(addr);
                        emit_instructions(program, ctx, [load_inst]);
                        Ok(IRExprBuildResult::Value(load_inst))
                    }
                }
            },
            PrimaryExpr::BracedExpr(expr) => expr.ir_buildable(program, ctx),
        }
    }
}
