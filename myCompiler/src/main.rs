use koopa::back::KoopaGenerator;
use lalrpop_util::lalrpop_mod;
use std::env::args;
use std::fs::read_to_string;
pub mod ast_def;
pub mod ir_build;
lalrpop_mod!(pub sysy);

//我们的命令行命令是cargo run -- -koopa hello.c -o hello.koopa
#[warn(unused_variables)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    //获取命令行参数
    let mut args = args();
    //跳过第一个参数，即程序名称
    args.next();
    //取第二个参数，即-koopa
    let mode = args.next().unwrap();
    //取第三个参数，即hello.c
    let input = args.next().unwrap();
    //忽略第四个参数,即-o
    args.next();
    //取第五个参数，即hello.koopa
    let output = args.next().unwrap();
    //读取输入文件
    let string_input = read_to_string(input)?;
    //调用lalrpop生成的parser解析输入文件
    let ast = sysy::CompUnitParser::new().parse(&string_input).expect("parse error");
    
    //生成内存形式的koopa IR
    let ir = match ir_build::build_ir(&ast) {
        Ok(program) => program,
        Err(msg) => return Err(msg.into()),
    };
    
    //将内存形式的IR转化为文本
    match mode.as_str() {
        "-koopa" => {
            let mut ir_to_text = KoopaGenerator::new(Vec::new());
            ir_to_text.generate_on(&ir).unwrap();
            std::fs::write(output, ir_to_text.writer())?;
            Ok(())
        },
        mode => Err(format!("Unsupported mode: {}", mode).into())
    }
}
