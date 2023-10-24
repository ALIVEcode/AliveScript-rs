#![allow(dead_code, unused_variables)]

use lalrpop_util::lalrpop_mod;

use crate::{lexer::Lexer, visitor::Visitable};

lalrpop_mod!(pub alivescript);

pub mod as_obj;
pub mod ast;
pub mod token;
mod lexer;
mod runner;
mod visitor;
mod data;

fn main() {
    let content = std::fs::read_to_string("./real.als").unwrap();
    let lexer = Lexer::new(&content[..]);
    let stmts = alivescript::ScriptParser::new().parse(lexer).unwrap();

    let mut visitor = runner::Runner::new();
    for stmt in stmts.iter() {
        stmt.accept(&mut visitor);
    }

    println!("{:#?}", stmts);

    println!("{:?}", visitor.get_datas());
}

#[cfg(test)]
mod test {
    #[test]
    fn nombres() {
    }

    #[test]
    fn texte() {
    }
}
