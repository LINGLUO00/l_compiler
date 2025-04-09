use crate::ast_def::stmt::*;
use crate::ast_def::expr::Expr;
use koopa::ir::{builder_traits::*, Program};

use super::{
    build_expr::{IRExprBuildResult, IRLeftValBuildResult},
    IRBuildResult, IRBuildable,
    context::*,
    append_basic_block, 
    create_basic_block, create_local_value, emit_instructions, 
};

impl IRBuildable for Stmt {
    type Output = IRBuildResult;
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        match &self {
            Stmt::UnmatchedStmt(unmatched_stmt) => unmatched_stmt.ir_buildable(program, ctx),
            Stmt::MatchedStmt(matched_stmt) => matched_stmt.ir_buildable(program, ctx),
        }
    }
}

impl IRBuildable for UnmatchedStmt {
    type Output = IRBuildResult;
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        self.NormalUnmatchedStmt.ir_buildable(program, ctx)
    }
}

impl IRBuildable for MatchedStmt {
    type Output = IRBuildResult;
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        self.NormalMatchedStmt.ir_buildable(program, ctx)
    }
}

impl IRBuildable for BasicStmt{
    type Output = IRBuildResult;
    fn ir_buildable(
        &self,
        program: &mut Program,
        ctx: &mut IRBuilderCtx,
    ) -> Result<Self::Output, String> {
        match &self {
            //1.assign：a=3;a[2]={1,3};
            BasicStmt::AssignStmt(left_val, init_exp) => {
                //处理左值：获取存储位置
                let result1 = left_val.ir_buildable(program, ctx)?;
                let left_val_ptr = match result1 {
                    IRLeftValBuildResult::Const(_) | IRLeftValBuildResult::TempVal(_) => {
                        return Err(format!(
                            "Constant expression or temp value ({:?}) should not be a left value!",
                            left_val
                        ))
                    }
                    IRLeftValBuildResult::Addr(addr) => addr,//得到左值在内存的位置
                };
                //处理初始化值，即右值
                let result2 = init_exp.ir_buildable(program, ctx)?;
                let rhs_value = match result2 {
                    IRExprBuildResult::Const(int) => {
                        create_local_value(program, ctx).integer(int)
                    }
                    IRExprBuildResult::Value(value) => value,
                };
                // 给左值赋值
                let store_inst = create_local_value(program, ctx)
                    .store(rhs_value, left_val_ptr);
                emit_instructions(program, ctx, [store_inst]);
                Ok(IRBuildResult::OK)
            }

            //2.Expr
            BasicStmt::Expr(e) => {
                if let Some(expr_val) = e {
                    expr_val.ir_buildable(program, ctx)?;
                    Ok(IRBuildResult::OK)
                } else {
                    Ok(IRBuildResult::OK)
                }
            }

            //3.Block
            BasicStmt::Block(block) => {
                ctx.symbol_table.add_table();
                block.ir_buildable(program, ctx)?;
                Ok(IRBuildResult::OK)
            }

            //4.If
            BasicStmt::IfStmt(bool_expr, stmt1, possible_stmt2) => {
                //处理条件表达式，如果是常量就直接创建出一个整数值，如果是变量就直接使用该值
                let bool_expr_value = match bool_expr.ir_buildable(program, ctx)? {
                    IRExprBuildResult::Const(int) => {
                        create_local_value(program, ctx).integer(int)
                    }
                    IRExprBuildResult::Value(value) => value,
                };
                //block_end为if语句结束的基本块，先创建出来而已，方便后面调用
                let block_end = create_basic_block(program, ctx, "if_block_end");
                //block_start为从ctx获取的当前基本块
                let block_start = ctx.current_block.expect("No current block. Should not happen! ");
                //if为true时的基本块为block1,在block1插入对应指令，block1结束后跳转到block_end
                let block1 = create_basic_block(program, ctx, "if_block_1");
                append_basic_block(program, ctx, [block1]);
                ctx.current_block = Some(block1);
                stmt1.ir_buildable(program, ctx)?;
                let jmp_inst = create_local_value(program, ctx).jump(block_end);
                emit_instructions(program, ctx, [jmp_inst]);

                // 如果有else分支，同样处理
                let jmp_block = match &**possible_stmt2 {
                    Some(stmt2) => {
                        let block2 = create_basic_block(program, ctx, "if_block_2");
                        append_basic_block(program, ctx, [block2]);
                        ctx.current_block = Some(block2);
                        stmt2.ir_buildable(program, ctx)?;
                        let jmp_inst = create_local_value(program, ctx).jump(block_end);
                        emit_instructions(program, ctx, [jmp_inst]);
                        block2
                    }
                    None => block_end,
                };
                //根据表达式的结果，如果为true就跳转到block1，否则跳转到jump_block(可能为else,可能为block_end)
                let if_stmt = create_local_value(program, ctx).branch(bool_expr_value, block1, jmp_block);
                ctx.current_block = Some(block_start);
                emit_instructions(program, ctx, [if_stmt]);

                // 处理if语句结束后的控制流，即if-else之后的语句
                ctx.current_block = Some(block_end);
                append_basic_block(program, ctx, [block_end]);

                Ok(IRBuildResult::OK)
            }

            //5.While
            BasicStmt::WhileStmt(bool_expr, stmt) => {
                let block_start = create_basic_block(program, ctx, "while_start");
                let block_body = create_basic_block(program, ctx, "while_body");
                let block_end = create_basic_block(program, ctx, "while_end");
                append_basic_block(
                    program,
                    ctx,
                    [block_start, block_body, block_end],
                );

                //发射跳转指令，跳转到while_start
                let start_jmp_inst = create_local_value(program, ctx).jump(block_start);
                emit_instructions(program, ctx, [start_jmp_inst]);
                // 获取表达式的bool,根据bool跳转到body或者end
                ctx.current_block = Some(block_start);
                let bool_expr_value = match bool_expr.ir_buildable(program, ctx)? {
                    IRExprBuildResult::Const(int) => {
                        create_local_value(program, ctx).integer(int)
                    }
                    IRExprBuildResult::Value(value) => value,
                };
                let branch_inst = create_local_value(program, ctx).branch(bool_expr_value, block_body, block_end);
                emit_instructions(program, ctx, [branch_inst]);

                //将 block_end 和 block_start 分别压入 break_tgt_blocks 和 continue_tgt_blocks，以支持 break 和 continue 语句
                ctx.current_block = Some(block_body);
                ctx.break_targets.push(block_end);
                ctx.continue_targets.push(block_start);
                //生成while body的IR
                match stmt.ir_buildable(program, ctx)? {
                    IRBuildResult::OK => {
                        let jmp_inst = create_local_value(program, ctx).jump(block_start);
                        emit_instructions(program, ctx, [jmp_inst]);
                    }
                    IRBuildResult::EARLYSTOPPING => {}
                    IRBuildResult::Error(err) => return Err(err)
                }
                //处理循环结束后的控制流，将控制流切换到 block_end，表示循环结束。弹出 break_tgt_blocks 和 continue_tgt_blocks，恢复之前的状态
                ctx.current_block = Some(block_end);
                ctx.break_targets.pop();
                ctx.continue_targets.pop();
                Ok(IRBuildResult::OK)
            }

            //6.Break
            BasicStmt::BreakStmt => {
                //获取break要跳转的目标块
                let tgt_block = match ctx.break_targets.last() {
                    Some(block) => Ok(block.clone()),
                    None => Err("Incorrect break statement!"),
                }?;
                //创建一条跳转指令，将控制流跳转到目标块（tgt_block），并将该指令插入到当前的中间表示（IR）中
                let jmp_inst = create_local_value(program, ctx).jump(tgt_block);
                emit_instructions(program, ctx, [jmp_inst]);
                Ok(IRBuildResult::EARLYSTOPPING)//返回 IRBuildResult::EARLYSTOPPING，表示当前控制流已经被中断，不需要继续生成后续的 IR 指令
            }

            //7.Continue
            BasicStmt::ContinueStmt => {
                let tgt_block = match ctx.continue_targets.last() {
                    Some(block) => Ok(block.clone()),
                    None => Err("Incorrect continue statement!"),
                }?;
                let jmp_inst = create_local_value(program, ctx).jump(tgt_block);
                emit_instructions(program, ctx, [jmp_inst]);
                Ok(IRBuildResult::EARLYSTOPPING)
            }

            //8.Return
            BasicStmt::ReturnStmt(returned_exp) => {
                let return_value = match returned_exp {
                    Some(exp) => Some({
                        let result = exp.ir_buildable(program, ctx)?;
                        match result {
                            IRExprBuildResult::Const(int) => {
                                create_local_value(program, ctx).integer(int)
                            }
                            IRExprBuildResult::Value(value) => value,
                        }
                    }),
                    None => None,
                };
                let return_stmt = create_local_value(program, ctx).ret(return_value);
                emit_instructions(program, ctx, [return_stmt]);
                Ok(IRBuildResult::EARLYSTOPPING)
            }
        }
    }
}