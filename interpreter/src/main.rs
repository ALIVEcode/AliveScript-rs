#![allow(dead_code, unused_variables)]

use std::{env, io::Write};

use data::{Data, Response};
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

struct IO {}

impl InterpretorIO for IO {
    fn send(&mut self, data: Data) {
        match data {
            Data::Afficher(s) => println!("{}", s),
            Data::Erreur { texte, ligne } => println!("{}", texte),
            Data::Demander { prompt } => todo!(),
        }
    }
    fn request(&mut self, data: Data) -> Option<Response> {
        match data {
            Data::Afficher(_) => todo!(),
            Data::Erreur { texte, ligne } => todo!(),
            Data::Demander { prompt } => {
                print!("{}", prompt.unwrap_or("Entrez une valeur: ".into()));
                std::io::stdout().flush().unwrap();
                let mut line = String::new();
                std::io::stdin().read_line(&mut line).unwrap();
                Some(Response::Text(line))
            }
        }
    }
}

fn main() {
    let mut io  = IO{};
    let script_file = env::args().nth(1).unwrap();
    let script = std::fs::read_to_string(script_file).unwrap();

    run_script(script, &mut io);
}

pub fn run_script<'a, IO: InterpretorIO + 'a>(script: String, interpretor_io: &mut IO) {
    let lexer = Lexer::new(&script[..]);
    let stmts = alivescript::ScriptParser::new().parse(lexer).unwrap();

    let mut visitor = runner::Runner::new(interpretor_io);
    visitor.visit_body(&stmts);
}
