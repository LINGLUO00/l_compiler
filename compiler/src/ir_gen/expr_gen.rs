use crate::ast_def::expr::*;
use koopa::ir::{builder_traits::*, Program, Value};
use super::*;

impl IRBuilder for Expr {
    fn build(
        &self,
        program: &mut Program,
        ir_gen_info: &mut IRGeneratorInfo,
    ) -> Result<IRBuildResult, String> {
        match self {
            Expr::Literal(lit) => lit.build(program, ir_gen_info),
        }
    }
}

impl IRBuilder for Literal {
    fn build(
        &self,
        program: &mut Program,
        _ir_gen_info: &mut IRGeneratorInfo,
    ) -> Result<IRBuildResult, String> {
        match self {
            Literal::Int(n) => {
                let value = program.new_value().integer(*n);
                Ok(IRBuildResult::Value(value))
            }
        }
    }
}
