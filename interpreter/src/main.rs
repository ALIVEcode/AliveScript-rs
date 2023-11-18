#![allow(dead_code, unused_variables)]

use std::{env, io::Write};

use alivescript_rust::{
    data::{Data, Response},
    io::InterpretorIO,
    run_script,
};

struct IO {}

impl InterpretorIO for IO {
    fn send(&mut self, data: Data) {
        match data {
            Data::Afficher(s) => println!("{}", s),
            Data::Erreur { texte, ligne } => println!("{}", texte),
            Data::Demander { prompt } => todo!(),
            Data::NotifInfo { msg } => todo!(),
            Data::NotifErr { msg } => todo!(),
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
            Data::NotifInfo { msg } => todo!(),
            Data::NotifErr { msg } => todo!(),
        }
    }
}

fn main() {
    let mut io = IO {};
    let script_file = env::args().nth(1).unwrap();
    let script = std::fs::read_to_string(script_file).unwrap();

    run_script(script, &mut io);
}
