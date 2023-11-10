#![allow(dead_code, unused_variables)]

use data::Data;
use io::InterpretorIO;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(alivescript, "/src/alivescript.rs");

mod as_modules;
pub(crate) mod as_obj;
pub(crate) mod ast;
pub mod data;
mod lexer;
mod runner;
pub(crate) mod token;
mod visitor;
pub mod io;

use crate::{lexer::Lexer, visitor::Visitor};

// fn main() {
//     let file = env::args().nth(1).expect("File to execute");
//     let content = std::fs::read_to_string(file).unwrap();
//     let lexer = Lexer::new(&content[..]);
//     let stmts = alivescript::ScriptParser::new().parse(lexer).unwrap();
//
//     let mut visitor = runner::Runner::new();
//     visitor.visit_body(&stmts);
//
//     let datas = visitor.get_datas();
//     // println!("{:#?}", visitor.get_datas());
//     exec(datas);
// }

pub fn run_script<'a, IO: InterpretorIO + 'a>(script: String, interpretor_io: &mut IO) {
    let lexer = Lexer::new(&script[..]);
    let stmts = alivescript::ScriptParser::new().parse(lexer).unwrap();

    let mut visitor = runner::Runner::new(interpretor_io);
    visitor.visit_body(&stmts);
}


fn exec(datas: Vec<Data>) {
    for data in datas {
        match data {
            Data::Afficher(obj) => println!("{}", obj),
            Data::Erreur { texte, ligne } => println!("{}", texte),
            Data::Demander { prompt } => todo!(),
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
