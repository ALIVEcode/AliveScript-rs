#![allow(dead_code, unused_variables, unused_imports)]

pub mod as_modules;
mod as_obj_utils;
#[cfg(feature = "py")]
mod as_py;
pub mod visitor;

pub mod as_obj;
pub mod ast;

pub mod lexer;
pub mod parser;
pub mod token;

pub mod data;
pub mod io;
pub mod runner;
pub mod utils;

pub mod bench;

mod compiler;

pub mod cli;

use std::rc::Rc;

use as_obj::ASErreur;
use ast::{Expr, Stmt};
use parser::build_ast_stmts;
use pest::pratt_parser::PrattParser;
use pest::Parser;
use pest_derive::Parser;

use crate::compiler::vm::VM;
use crate::compiler::Compiler;
use crate::data::Data;
use crate::io::InterpretorIO;
use crate::runner::Runner;

#[derive(Parser)]
#[grammar = "./alivescript.pest"]
struct AlivescriptParser;

use crate::visitor::Visitor;

pub fn get_err_line(script: &String, start: usize, end: usize) -> (String, usize) {
    let line_num = script[0..end].split('\n').count();

    let line_start = script[0..end].rfind(&['\n', ';']).unwrap_or(0);
    let line_end = script[start..]
        .find(&['\n', ';'])
        .map(|i| i + start)
        .unwrap_or(script.len());
    (script[line_start..=line_end].trim().to_owned(), line_num)
}

pub fn run_script<'a, IO: InterpretorIO + 'a>(script: &String, interpretor_io: &mut IO) {
    // let lexer = Lexer::new(&script[..]);
    let result_stmts = AlivescriptParser::parse(Rule::script, script);

    match result_stmts {
        Ok(stmts) => {
            let mut visitor = Runner::new(interpretor_io);
            let stmts = build_ast_stmts(stmts).unwrap();
            visitor.visit_body(&stmts);
        }
        Err(err) => interpretor_io.send(Data::Erreur {
            texte: format!("ErreurSyntaxe: {}", err.to_string()),
            ligne: 0,
        }),
    };
}

pub fn run_script_from_file<'a, IO: InterpretorIO + 'a>(
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
            let mut visitor = Runner::new_with_file(interpretor_io, script_file);
            let stmts = build_ast_stmts(stmts).unwrap();
            visitor.visit_body(&stmts);
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

pub fn run_script_with_runner<'a>(
    script: &String,
    runner: &mut Runner<'a>,
) -> Result<(), pest::error::Error<Rule>> {
    let debug = script.starts_with(":d");
    let script = script.trim_start_matches(":d");

    let stmts = AlivescriptParser::parse(Rule::script, script)?;
    if debug {
        println!("{:#?}", stmts);
    }

    let stmts = build_ast_stmts(stmts)?;
    runner.visit_body(&stmts);
    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn nombres() {}

    #[test]
    fn texte() {}
}

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
            let compiler = Compiler::new(script);
            let closure = compiler.compile(stmts).unwrap();
            let mut vm = VM::new();
            let result = vm.run(Rc::new(closure)).unwrap();
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
