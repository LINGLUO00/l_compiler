use crate::ast_def::stmt::*;
use koopa::ir::{builder_traits::*, Program};
use super::*;



impl IRBuilder for Stmt {
    fn build(
        &self,
        program: &mut Program,
        ir_gen_info: &mut IRGeneratorInfo,
    ) -> Result<IRBuildResult, String> {
        match &self {
            Stmt::MatchedStmt(stmt) => stmt.build(program, ir_gen_info),
        }
    }
}

impl IRBuilder for MatchedStmt {
    fn build(
        &self,
        program: &mut Program,
        ir_gen_info: &mut IRGeneratorInfo,
    ) -> Result<IRBuildResult, String> {
        self.default.build(program, ir_gen_info)
    }
}

impl IRBuilder for BasicStmt {
    fn build(
        &self,
        program: &mut Program,
        ir_gen_info: &mut IRGeneratorInfo,
    ) -> Result<IRBuildResult, String> {
        match self {
            BasicStmt::ReturnStmt(expr) => {
                let value = expr.build(program, ir_gen_info)?;
                // 通过GlobalBuilder直接创建return指令
                // 手动创建return指令
                let ret_value = {
                    let mut program_mut = program.borrow_mut();
                    let ret = program_mut.new_value_data(
                        koopa::ir::ValueData::new_instruction(
                            koopa::ir::Instruction::new_ret(Some(value))
                        )
                    );
                    ret
                };
                Ok(IRBuildResult::Value(ret_value))
            }
        }
    }
}
