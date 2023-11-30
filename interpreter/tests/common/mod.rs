#![allow(dead_code, unused_variables)]

use std::{path::Path, vec::IntoIter};

use alivescript_rust::{
    data::{Data, Response},
    io::InterpretorIO,
    run_script,
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
            Data::GetFichier(path) => {
                let content = std::fs::read_to_string(path).unwrap();
                Some(Response::Text(content))
            }
            Data::NotifInfo { msg } => todo!(),
            Data::NotifErr { msg } => todo!(),
        }
    }
}

pub fn run_test<P>(file_path: P, expected: &Vec<Data>)
where
    P: AsRef<Path>,
{
    let script = std::fs::read_to_string(file_path).unwrap();

    let mut test_io = TestIO::default();
    run_script(script, &mut test_io);

    assert_eq!(test_io.outputs(), expected);
}
