//use std::result;
use koopa::ir::{builder_traits::*, FunctionData, Program, Type, Value};
use crate::ast_def::decl::*;
use super::{IRBuilder, IRBuildResult, IRGeneratorInfo};

impl IRBuilder for FuncDef{
    fn build (&self, program:&mut Program, ir_gen_info:&mut IRGeneratorInfo)->Result<IRBuildResult,String>
    {
        let FuncDef::Default(func_type, func_ident, block)=self;
        let func_type=Type::get(func_type.ret_ty.clone());
        let func = program.new_func(FunctionData::with_param_names(func_ident.ident.clone(), vec![], func_type));
        let func_data = program.func_mut(func);
        let new_block = func_data.dfg_mut().new_bb().basic_block(None);
        func_data.layout_mut().bbs_mut().extend([new_block]);
        ir_gen_info.curr_block = Some(new_block);
        ir_gen_info.curr_func = Some(func);

        block.build(program, ir_gen_info)?;
        ir_gen_info.curr_func = None;
        ir_gen_info.curr_block = None;
        Ok(IRBuildResult::OK)
    }
}

impl IRBuilder for Block {
    fn build(&self,program:&mut Program, ir_gen_info:&mut IRGeneratorInfo) -> Result<IRBuildResult, String> {
        let Block::Default(stmts)=self;
        for stmt in stmts {
            stmt.build(program, ir_gen_info)?;
        }
        Ok(IRBuildResult::OK)
    }
}

impl IRBuilder for BlockItem {
    fn build(
        &self,
        program: &mut Program,
        ir_gen_info: &mut IRGeneratorInfo,
    ) -> Result<IRBuildResult, String> {
        match self {
            BlockItem::Stmt(stmt) => stmt.build(program, ir_gen_info),
        }
    }
}
