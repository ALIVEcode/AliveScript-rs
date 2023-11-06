#![allow(dead_code, unused_variables)]

use std::env;

use data::Data;
use lalrpop_util::lalrpop_mod;

use crate::{lexer::Lexer, visitor::Visitor};

lalrpop_mod!(pub alivescript);

pub mod as_obj;
pub mod ast;
mod data;
mod lexer;
mod runner;
pub mod token;
mod visitor;
mod as_modules;

fn main() {
    let file = env::args().nth(1).expect("File to execute");
    let content = std::fs::read_to_string(file).unwrap();
    let lexer = Lexer::new(&content[..]);
    let stmts = alivescript::ScriptParser::new().parse(lexer).unwrap();

    let mut visitor = runner::Runner::new();
    visitor.visit_body(&stmts);

    let datas = visitor.get_datas();
    // println!("{:#?}", visitor.get_datas());
    exec(datas);
}

fn exec(datas: Vec<Data>) {
    for data in datas {
        match data {
            Data::Afficher(obj) => println!("{}", obj),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn nombres() {}

    #[test]
    fn texte() {}
}
