#![allow(dead_code, unused_variables)]

use std::vec::IntoIter;

use alivescript_rust::{
    data::{Data, Response},
    io::InterpretorIO,
};

#[derive(Default)]
pub struct TestIO {
    inputs: IntoIter<Response>,
    outputs: Vec<Data>,
}

impl TestIO {
    pub fn outputs(&self) -> &Vec<Data> {
        &self.outputs
    }
    
}

impl InterpretorIO for TestIO {
    fn send(&mut self, data: Data) {
        self.outputs.push(data);
    }

    fn request(&mut self, data: Data) -> Option<Response> {
        match data {
            Data::Afficher(_) => todo!(),
            Data::Erreur { texte, ligne } => todo!(),
            Data::Demander { prompt } => {
                self.outputs.push(Data::Afficher(
                    prompt.unwrap_or("Entrez une valeur: ".into()),
                ));
                self.inputs.next()
            }
            Data::NotifInfo { msg } => todo!(),
            Data::NotifErr { msg } => todo!(),
        }
    }
}



