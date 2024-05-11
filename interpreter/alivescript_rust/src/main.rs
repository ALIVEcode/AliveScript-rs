#![allow(dead_code, unused_variables)]
use lalrpop_util::ParseError;

use std::{cell::RefCell, env, io::Write, rc::Rc};

use alivescript_rust::{
    data::{Data, Response},
    get_err_line,
    io::InterpretorIO,
    run_script_from_file, run_script_with_runner,
};

const ALIVESCRIPT_VERSION: &'static str = "0.8.0";

struct IO {}

impl InterpretorIO for IO {
    fn send(&mut self, data: Data) {
        match data {
            Data::Afficher(s) => println!("{}", s),
            Data::Erreur { texte, ligne } => eprintln!("{}", texte),
            _ => todo!(),
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
            Data::GetFichier(file_path) => {
                let content = std::fs::read_to_string(file_path).ok()?;
                Some(Response::Text(content))
            }
            Data::NotifInfo { msg } => todo!(),
            Data::NotifErr { msg } => todo!(),
        }
    }
}

struct ReplIO {
    console: Rc<RefCell<console::Term>>,
}

impl InterpretorIO for ReplIO {
    fn send(&mut self, data: Data) {
        match data {
            Data::Afficher(s) => writeln!(self.console.borrow_mut(), "{}", s).unwrap(),
            Data::Erreur { texte, ligne } => {
                writeln!(self.console.borrow_mut(), "{}", texte).unwrap()
            }
            _ => todo!(),
        }
    }
    fn request(&mut self, data: Data) -> Option<Response> {
        match data {
            Data::Afficher(_) => todo!(),
            Data::Erreur { texte, ligne } => todo!(),
            Data::Demander { prompt } => {
                let line = self
                    .console
                    .borrow_mut()
                    .read_line_initial_text(prompt.unwrap_or("Entrez une valeur: ".into()).as_str())
                    .unwrap();
                Some(Response::Text(line))
            }
            Data::GetFichier(file_path) => {
                let content = std::fs::read_to_string(file_path).ok()?;
                Some(Response::Text(content))
            }
            Data::NotifInfo { msg } => todo!(),
            Data::NotifErr { msg } => todo!(),
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut args = env::args();
    let first_arg = args.nth(1);
    if let Some(script_file) = first_arg {
        if script_file == "--version" {
            println!("{}", ALIVESCRIPT_VERSION);
            return Ok(());
        }
        let mut io = IO {};
        let script = std::fs::read_to_string(&script_file).unwrap();
        run_script_from_file(&script, &mut io, script_file);
        return Ok(());
    }
    let console = Rc::new(RefCell::new(console::Term::stdout()));
    let mut io = ReplIO {
        console: Rc::clone(&console),
    };
    let mut runner = alivescript_rust::runner::Runner::new(&mut io);
    loop {
        write!(console.borrow_mut(), "> ")?;
        let mut in_block = false;
        let mut body = String::new();
        loop {
            let line = console.borrow_mut().read_line()?;
            if in_block {
                console.borrow_mut().clear_last_lines(1)?;
                writeln!(console.borrow_mut(), "{}", line.replace("\t", "  "))?;
            }
            body += &(line + "\n");
            if let Err(err) = run_script_with_runner(&body, &mut runner) {
                match err {
                    ParseError::UnrecognizedToken { token, expected } => {
                        let (line, line_num) = get_err_line(&body, token.0, token.2);
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!(
                                "À la ligne {} ('{}'). Jeton non reconnu: {}. Jetons valides dans cette position: {}", 
                                line_num, 
                                line, 
                                token.1,
                                expected.join(", ")
                            ),
                        ));
                    }

                    ParseError::UnrecognizedEof { location, expected } => {
                        body += "\n";
                        write!(console.borrow_mut(), "|")?;
                        in_block = true;
                        continue;
                    }
                    ParseError::InvalidToken { location } => todo!(),
                    ParseError::ExtraToken { token } => todo!(),
                    ParseError::User { error } => todo!(),
                }
            } else {
                break;
            }
        }
        runner.remove_error_status();
        if let Some(value) = runner.get_stmt_result() {
            writeln!(console.borrow_mut(), "{}", value.repr())?;
        }
    }
}
