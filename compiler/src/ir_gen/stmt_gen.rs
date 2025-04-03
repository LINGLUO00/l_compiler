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
        match &self {
        BasicStmt::ReturnStmt(returned_exp) => {
            let return_value = match returned_exp {
                Some(exp) => Some({
                    let result = exp.build(program, my_ir_generator_info)?; // Build the returned Exp into curr_value.
                    match result {
                        IRExpBuildResult::Const(int) => {
                            create_new_local_value(program, my_ir_generator_info).integer(int)
                        }
                        IRExpBuildResult::Value(value) => value,
                    }
                }),
                None => None,
            };
            let return_stmt =
                new_local_value_builder(program, my_ir_generator_info).ret(return_value);
            insert_local_instructions(program, my_ir_generator_info, [return_stmt]);
            Ok(IRBuildResult::EARLYSTOPPING)
         }
        }
    }
}