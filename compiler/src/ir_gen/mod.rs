pub mod decl_gen;
pub mod stmt_gen;
pub mod expr_gen;
use std::collections::HashMap;
use koopa::ir::{Program, FunctionData, Type, BasicBlock, Function};
use crate::ast_def::*;


pub struct IRGeneratorInfo{
    curr_block:Option<BasicBlock>,
    curr_func:Option<Function>,
    bb_count:usize,
    function_table:HashMap<String, Function>,
}

pub fn generate_ir(comp_unit: &CompUnit) -> Result<Program, String> {
    //ir_gen的流程：program -> function -> block -> stmt
    let mut program = Program::new();
    let mut ir_generator_info = IRGeneratorInfo {
        curr_block: None,
        curr_func: None,
        bb_count:0,
        function_table:HashMap::new(),
    };
    comp_unit.build(&mut program, &mut ir_generator_info)?;
    Ok(program)
}

// impl IRGeneratorInfo{
//     //检查全局符号（如函数名，变量名）是否已经存在
// }

pub enum IRBuildResult{
    OK,
    Value(koopa::ir::Value),
    Error(String),//TODO:在error.rs中定义错误类型
}

pub trait IRBuilder{
    fn build(&self,program:&mut Program, ir_gen_info:&mut IRGeneratorInfo) -> Result<IRBuildResult, String>;
}

impl IRBuilder for CompUnit{
    fn build(&self,program:&mut Program, ir_gen_info:&mut IRGeneratorInfo) -> Result<IRBuildResult, String> {
        let CompUnit::Default(units)=self;
        for unit in units{
            unit.build(program,ir_gen_info)?;
        }
        Ok(IRBuildResult::OK)
    }
}

impl IRBuilder for Unit{
    fn build(&self,program:&mut Program, ir_gen_info:&mut IRGeneratorInfo) -> Result<IRBuildResult, String> {
    let Unit::FuncDef(func_def) =self;
    func_def.build(program, ir_gen_info)?;
    Ok(IRBuildResult::OK)
    }
}
