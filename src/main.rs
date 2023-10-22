#![allow(dead_code)]

use lalrpop_util::lalrpop_mod;

use crate::lexer::Lexer;

lalrpop_mod!(pub alivescript);

pub mod as_obj;
pub mod ast;
pub mod token;
mod lexer;

fn main() {
    let content = std::fs::read_to_string("./test2.als").unwrap();
    let lexer = Lexer::new(&content[..]);
    let result = alivescript::ScriptParser::new().parse(lexer).unwrap();
    println!("{:#?}", result)
}

#[cfg(test)]
mod test {
    use super::alivescript;
    use super::as_obj::ASObj;
    use super::ast::Expr;

    #[test]
    fn nombres() {
    }

    #[test]
    fn texte() {
    }
}
