use lalrpop_util::lalrpop_mod;//引入 LALRPOP 解析器生成工具的相关功能，lalrpop_mod! 宏用于生成语法解析器模块，需要配合 .lalrpop 语法文件使用，自动生成 Rust 解析代码
use std::env::args;//引入标准库中的命令行参数处理模块
use std::fs::read_to_string;//引入标准库中的文件处理模块，read_to_string("path") 会一次性读取整个文件到字符串
use std::io::Result;//用于文件操作、网络操作等可能失败的 I/O 操作的错误处理
pub mod ast_def;//引入自定义的 AST 定义模块 ast_def

lalrpop_mod!(pub sysy);

//我们的命令行命令是cargo run -- -koopa hello.c -o hello.lalrpop
#[warn(unused_variables)]
fn main() -> Result<()>{
    //获取命令行参数
    let mut args=args();
    //跳过第一个参数，即--
    args.next();
    //取第二个参数，即-koopa
    let _mode = args.next().unwrap();
    //取第三个参数，即hello.c
    let input = args.next().unwrap();
    //忽略第四个参数,即-o
    args.next();
    //取第五个参数，即hello.lalrpop
    let _output = args.next().unwrap();
    //读取输入文件
    let string_input = read_to_string(input)?;
    //调用lalrpop生成的parser解析输入文件
    let ast = sysy::CompUnitParser::new().parse(&string_input).expect("parse error");
    //输出解析得到的AST,其中:表示结构体的字段名，#表示格式化输出，？表示自动推导类型
    std::fs::write(_output, format!("{:#?}", ast))?;
    return Ok(());

    
}
