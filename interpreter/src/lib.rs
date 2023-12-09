#![allow(dead_code, unused_variables)]

mod as_modules;
mod visitor;
mod as_py;
mod as_obj_utils;

pub(crate) mod as_obj;
pub(crate) mod ast;
pub(crate) mod lexer;
pub(crate) mod token;

pub mod data;
pub mod io;
pub mod runner;

use crate::data::Data;
use crate::io::InterpretorIO;
use crate::runner::Runner;
use crate::lexer::LexicalError;
use lalrpop_util::lalrpop_mod;
use lalrpop_util::ParseError;

lalrpop_mod!(pub alivescript, "/src/alivescript.rs");

use crate::{lexer::Lexer, visitor::Visitor};

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
    let lexer = Lexer::new(&script[..]);
    let result_stmts = alivescript::ScriptParser::new().parse(lexer);

    match result_stmts {
        Ok(stmts) => {
            let mut visitor = Runner::new(interpretor_io);
            visitor.visit_body(&stmts);
        }
        Err(err) => {
            let err_txt = match err {
                ParseError::UnrecognizedToken { token, expected } => {
                    let (line, line_num) = get_err_line(&script, token.0, token.2);
                    format!("À la ligne {} ('{}'). Jeton non reconnu: {}. Jetons valides dans cette position: {}",
                             line_num, line, token.1, expected.join(", "))
                }
                ParseError::InvalidToken { location } => todo!(),
                ParseError::UnrecognizedEof { location, expected } => todo!(),
                ParseError::ExtraToken { token } => todo!(),
                ParseError::User { error } => todo!(),
            };
            interpretor_io.send(Data::Erreur {
                texte: format!("ErreurSyntaxe: {}", err_txt),
                ligne: 0,
            })
        }
    };
}

pub fn run_script_from_file<'a, IO: InterpretorIO + 'a>(script: &String, interpretor_io: &mut IO, script_file: String) {
    let lexer = Lexer::new(&script[..]);
    let result_stmts = alivescript::ScriptParser::new().parse(lexer);

    match result_stmts {
        Ok(stmts) => {
            let mut visitor = Runner::new_with_file(interpretor_io, script_file);
            visitor.visit_body(&stmts);
        }
        Err(err) => {
            let err_txt = match err {
                ParseError::UnrecognizedToken { token, expected } => {
                    let (line, line_num) = get_err_line(&script, token.0, token.2);
                    format!("À la ligne {} ('{}'). Jeton non reconnu: {}. Jetons valides dans cette position: {}",
                             line_num, line, token.1, expected.join(", "))
                }
                ParseError::InvalidToken { location } => todo!(),
                ParseError::UnrecognizedEof { location, expected } => todo!(),
                ParseError::ExtraToken { token } => todo!(),
                ParseError::User { error } => todo!(),
            };
            interpretor_io.send(Data::Erreur {
                texte: format!("ErreurSyntaxe: {}", err_txt),
                ligne: 0,
            })
        }
    };
}
pub fn run_script_with_runner<'a>(
    script: &String,
    runner: &mut Runner<'a>,
) -> Result<(), ParseError<usize, token::Token, LexicalError>> {
    let lexer = Lexer::new(script.as_str());
    let result_stmts = alivescript::ScriptParser::new().parse(lexer);

    match result_stmts {
        Ok(stmts) => {
            runner.visit_body(&stmts);
            Ok(())
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn nombres() {}

    #[test]
    fn texte() {}
}
