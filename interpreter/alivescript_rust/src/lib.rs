#![allow(unused)]

// pub mod as_modules;
pub mod as_obj_utils;
#[cfg(feature = "py")]
mod as_py;

pub mod data;
pub mod io;
pub mod utils;

pub mod bench;

pub mod compiler;
pub mod runtime;

pub mod cli;

use pest::Parser;
use pest_derive::Parser;

use crate::compiler::Compiler;
use crate::data::Data;
use crate::io::InterpretorIO;
use crate::runtime::vm::VM;

#[derive(Parser)]
#[grammar = "./alivescript.pest"]
pub struct AlivescriptParser;

pub fn compile_script_from_file2<'a, IO: InterpretorIO + 'a>(
    script: &String,
    interpretor_io: &mut IO,
    script_file: String,
) {
    let debug = script.starts_with("#debug!");
    let result_stmts = AlivescriptParser::parse(Rule::script, script);
    if debug {
        println!("{:#?}", result_stmts);
    }

    match result_stmts {
        Ok(stmts) => {
            // let mut visitor = Runner::new_with_file(interpretor_io, script_file);
            let compiler = Compiler::new(script, script_file.clone());
            let closure = compiler.compile(stmts).unwrap();
            let mut vm = VM::new(script_file);
            let result = vm.run(closure).unwrap();
            // println!("{:#?}", vm.stack);
            // println!("{:?}", result);
        }
        Err(err) => interpretor_io.send(Data::Erreur {
            texte: format!(
                "ErreurSyntaxe: {}\n{:#?}",
                err.to_string(),
                err.parse_attempts()
            ),
            ligne: 0,
        }),
    };
}
