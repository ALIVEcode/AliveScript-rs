use core::fmt;
use std::{
    any::Any,
    fs,
    io::{self, BufRead, BufReader, Read, Write},
    ops::Deref,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use crate::{
    as_module, as_module_fonction,
    compiler::{
        obj::{Function, Value},
        value::{ArcNativeObjet, NativeMethod, NativeObjet, Type},
    },
    runtime::err::RuntimeError,
    stdlib::{LazyModule, path::ASPath},
    unpack, unpack_native,
};

trait ReadDebug: io::Read + fmt::Debug {}
trait WriteDebug: io::Write + fmt::Debug {}

impl WriteDebug for io::Stdout {}
impl WriteDebug for io::Stderr {}
impl ReadDebug for fs::File {}
impl WriteDebug for fs::File {}

#[derive(Debug)]
struct Reader {
    reader: Arc<RwLock<dyn ReadDebug>>,
}

impl NativeObjet for Reader {
    fn type_name(&self) -> &'static str {
        "ES.Lecteur"
    }

    fn get_member(
        self: Arc<Self>,
        vm: &mut crate::runtime::vm::VM,
        name: &str,
    ) -> Result<Value, crate::runtime::err::RuntimeError> {
        let es = vm.get_std_module("ES");
        match es.read().unwrap().get_member(name)? {
            Value::Function(Function::NativeFunction(function)) => {
                Ok(Value::Function(Function::NativeMethod(NativeMethod {
                    func: function,
                    inst_value: Box::new(Value::NativeObjet(self)),
                })))
            }
            v => Ok(v),
        }
    }

    fn as_any(self: Arc<Self>) -> Arc<dyn Any> {
        self
    }
}

#[derive(Debug)]
struct Writer {
    writer: Arc<RwLock<dyn WriteDebug>>,
}

impl NativeObjet for Writer {
    fn type_name(&self) -> &'static str {
        "ES.Scripteur"
    }

    fn get_member(
        self: Arc<Self>,
        vm: &mut crate::runtime::vm::VM,
        name: &str,
    ) -> Result<Value, crate::runtime::err::RuntimeError> {
        let es = vm.get_std_module("ES");
        match es.read().unwrap().get_member(name)? {
            Value::Function(Function::NativeFunction(function)) => {
                Ok(Value::Function(Function::NativeMethod(NativeMethod {
                    func: function,
                    inst_value: Box::new(Value::NativeObjet(self)),
                })))
            }
            v => Ok(v),
        }
    }

    fn as_any(self: Arc<Self>) -> Arc<dyn Any> {
        self
    }
}

as_module! {
    module ES {}

    fn load(&self) {
        [
            as_module_fonction! {
                existe(filename: Type::Texte): Type::Booleen => {
                    let filename = filename.as_texte().unwrap();
                    Ok(Some(Value::Booleen(fs::exists(filename).unwrap_or(false))))
                }
            },
            as_module_fonction! {
                sortieStd() => {
                    let fh = Writer { writer: Arc::new(RwLock::new(io::stdout())) };
                    Ok(Some(Value::NativeObjet(Arc::new(fh))))
                }
            },
            as_module_fonction! {
                sortieErr() => {
                    let fh = Writer { writer: Arc::new(RwLock::new(io::stderr())) };
                    Ok(Some(Value::NativeObjet(Arc::new(fh))))
                }
            },
            as_module_fonction! {
                ouvrir(
                    filename: Type::union_of(Type::Texte, Type::Objet(String::from("Chemin.Chemin"))),
                    mode: Type::Texte
                ): Type::Custom => {
                    unpack_native!(filename: &ASPath = filename => {
                        filename.0.clone()
                    } else {
                        PathBuf::from(filename.as_texte()?)
                    });

                    let filename = &filename.display().to_string();

                    let mode = mode.as_texte().unwrap();
                    match mode {
                        "écriture" | "ecriture" | "e" | "é" => {
                            let file = fs::File::create(filename)
                                .map_err(|err| RuntimeError::generic_err(format!(
                                    "Erreur lors de l'ouverture du fichier '{}'\n{}", filename, err
                                )))?;

                            Ok(Some(Value::NativeObjet(Arc::new(Writer { writer: Arc::new(RwLock::new(file)) }))))
                        }
                        "ajout" | "a" => {
                            let file = fs::File::options().append(true).open(filename)
                                .map_err(|err| RuntimeError::generic_err(format!(
                                    "Erreur lors de l'ouverture du fichier '{}'\n{}", filename, err
                                )))?;

                            Ok(Some(Value::NativeObjet(Arc::new(Writer { writer: Arc::new(RwLock::new(file)) }))))
                        }
                        "lecture" | "l" => {
                            let file = fs::File::open(filename)
                                .map_err(|err| RuntimeError::generic_err(format!(
                                    "Erreur lors de l'ouverture du fichier '{}'\n{}", filename, err
                                )))?;

                            Ok(Some(Value::NativeObjet(Arc::new(Reader {reader: Arc::new(RwLock::new(file))}))))
                        }
                        _ => return Err(RuntimeError::generic_err(format!("Mode d'ouverture invalide '{}'", mode)))
                    }
                }
            },
            as_module_fonction! {
                créerDossier(
                    chemin: Type::union_of(Type::Texte, Type::Objet(String::from("Chemin.Chemin"))),
                ) => {
                    unpack_native!(chemin: &ASPath = chemin => {
                        chemin.0.clone()
                    } else {
                        PathBuf::from(chemin.as_texte()?)
                    });

                    fs::create_dir_all(&chemin).map_err(|e|
                        RuntimeError::generic_err(format!(
                            "Erreur lors de la création du dossier '{}'\n{}", chemin.display(), e
                        )))?;

                    Ok(Some(Value::Nul))
                }
            },
            as_module_fonction! {
                écrire(inst: Type::Objet(String::from("ES.Scripteur")), msg: Type::Texte): Type::Entier => {
                    unpack_native!(f: &Writer = inst);

                    let msg = msg.as_texte().unwrap();
                    let nb_bytes = f.writer.write().unwrap().write(msg.as_bytes()).unwrap();

                    Ok(Some(Value::Entier(nb_bytes as i64)))
                }
            },
            as_module_fonction! {
                écrireLigne(inst: Type::Objet(String::from("ES.Scripteur")), msg: Type::Texte): Type::Entier => {
                    unpack_native!(f: &Writer = inst);

                    let msg = String::from(msg.as_texte().unwrap()) + "\n";
                    let nb_bytes = f.writer.write().unwrap().write(msg.as_bytes()).unwrap();

                    Ok(Some(Value::Entier(nb_bytes as i64)))
                }
            },
            as_module_fonction! {
                lireLigne(inst: Type::Objet(String::from("ES.Lecteur"))): Type::Texte => {
                    unpack_native!(f: &Reader = inst);

                    let mut file = f.reader.write().unwrap();
                    let mut line = String::new();
                    loop {
                        let mut buffer = [0; 1];
                        let result = file.read_exact(&mut buffer);
                        match result {
                            Ok(_) => {}
                            Err(err) => {
                                if err.kind() == io::ErrorKind::UnexpectedEof {
                                    break;
                                }
                                return Err(RuntimeError::generic_err(format!("Erreur lors de la lecture du fichier:\n{}", err)))
                            }
                        }
                        line.push(buffer[0] as char);
                        if buffer[0] as char == '\n' {
                            break;
                        }
                    }

                    Ok(Some(Value::Texte(line)))
                }
            },
            as_module_fonction! {
                lignes(inst: Type::Objet(String::from("ES.Lecteur"))): Type::Liste => {
                    unpack_native!(f: &Reader = inst);

                    let mut file = f.reader.write().unwrap();
                    let mut s = String::new();
                    let result = file.read_to_string(&mut s);
                    match result {
                        Ok(_) => {}
                        Err(err) => return Err(RuntimeError::generic_err(format!(
                            "Erreur lors de la lecture du fichier:\n{}", err
                        )))
                    };

                    Ok(Some(Value::liste(s.lines().map(|line| Value::Texte(line.to_string())).collect())))
                }
            },
            as_module_fonction! {
                lireTout(inst: Type::Objet(String::from("ES.Lecteur"))): Type::Texte => {
                    unpack_native!(f: &Reader = inst);

                    let mut file = f.reader.write().unwrap();
                    let mut s = String::new();
                    let result = file.read_to_string(&mut s);
                    match result {
                        Ok(_) => {}
                        Err(err) => return Err(RuntimeError::generic_err(format!(
                            "Erreur lors de la lecture du fichier:\n{}", err
                        )))
                    };

                    Ok(Some(Value::Texte(s)))
                }
            },
            as_module_fonction! {
                lire(inst: Type::Objet(String::from("ES.Lecteur")), nbcars: Type::Entier): Type::Texte => {
                    unpack_native!(f: &Reader = inst);

                    let mut file = f.reader.write().unwrap();
                    let mut s = vec![0; nbcars.as_entier().unwrap() as usize];
                    let result = file.read(&mut s);
                    let nb = match result {
                        Ok(nb) => nb,
                        Err(err) => return Err(RuntimeError::generic_err(format!(
                            "Erreur lors de la lecture du fichier:\n{}", err
                        )))
                    };

                    Ok(Some(Value::Texte(String::from_utf8(s[..nb].to_vec()).map_err(|err| {
                        RuntimeError::generic_err(format!("Erreur lors de la conversion en texte:\n{}", err))
                    })?)))
                }
            },
        ]
    }
}
