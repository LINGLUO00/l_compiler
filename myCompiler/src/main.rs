use koopa::back::KoopaGenerator;
use lalrpop_util::lalrpop_mod;//引入 LALRPOP 解析器生成工具的相关功能，lalrpop_mod! 宏用于生成语法解析器模块，需要配合 .lalrpop 语法文件使用，自动生成 Rust 解析代码
use std::env::args;//引入标准库中的命令行参数处理模块
use std::fs::read_to_string;//引入标准库中的文件处理模块，read_to_string("path") 会一次性读取整个文件到字符串
use std::io::Result;//用于文件操作、网络操作等可能失败的 I/O 操作的错误处理
pub mod ast_def;//引入自定义的 AST 定义模块 ast_def
pub mod ir_build;//引入自定义的 IR 构建模块 ir_build
lalrpop_mod!(pub sysy);

//我们的命令行命令是cargo run -- -koopa hello.c -o hello.lalrpop
#[warn(unused_variables)]
fn main() -> Result<(),Box<dyn std::error::Error>> {
    //获取命令行参数
    let mut args=args();
    //跳过第一个参数，即--
    args.next();
    //取第二个参数，即-koopa
    let mode = args.next().unwrap();
    //取第三个参数，即hello.c
    let input = args.next().unwrap();
    //忽略第四个参数,即-o
    args.next();
    //取第五个参数，即hello.lalrpop
    let output = args.next().unwrap();
    //读取输入文件
    let string_input = read_to_string(input)?;
    //调用lalrpop生成的parser解析输入文件
    //let ast = sysy::CompUnitParser::new().parse(&string_input).expect("parse error");
    //输出解析得到的AST,其中:表示结构体的字段名，#表示格式化输出，？表示自动推导类型
    std::fs::write(output, format!("{:#?}", ast))?;
    //生成内存形式的kooap IR
    let ir:koopa::ir::Program = ir_build::build_ir(&ast);
    //将内存形式的IR转化为文本
    match mode.as_str(){
        "-koopa"=>{
            let mut ir_to_text = KoopaGenerator::new(Vec::new());
            IR_to_text.generate_on(&ir).unwrap();
            std::fs::write(output, ir_to_text.writer())?;
            return Ok(());
        }
        mode => Err(mode)
    }

    return Ok(());

    
}
