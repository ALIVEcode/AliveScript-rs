use core::time;
use std::{
    any::Any,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, LazyLock, RwLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use dyn_fmt::AsStrFormatExt;
use rand::random_range;
use uuid::timestamp;

use crate::{
    as_module, as_module_fonction,
    compiler::{
        bytecode::BinOpcode,
        obj::Function,
        value::{NativeMethod, NativeObjet},
    },
    runtime::{
        config::{PermissionSet, VMAction, VMConfig},
        vm::VM,
    },
    unpack, unpack_native,
};
use crate::{
    compiler::{
        obj::Value,
        value::{ASModule, ArcModule, Type},
    },
    runtime::err::RuntimeError,
};

#[derive(Debug)]
pub struct ASPath(pub PathBuf);

impl NativeObjet for ASPath {
    fn type_name(&self) -> &'static str {
        "Chemin.Chemin"
    }

    fn get_member(
        self: Arc<Self>,
        vm: &mut crate::runtime::vm::VM,
        name: &str,
    ) -> Result<Value, crate::runtime::err::RuntimeError> {
        let chemin = vm.get_std_module("Chemin");
        let es = vm.get_std_module("ES");
        match chemin.read().unwrap().get_member(name) {
            Ok(Value::Function(Function::NativeFunction(function))) => {
                return Ok(Value::Function(Function::NativeMethod(NativeMethod {
                    func: function,
                    inst_value: Box::new(Value::NativeObjet(self)),
                })));
            }
            Ok(v) => Ok(v),
            Err(_) => match es.read().unwrap().get_member(name)? {
                Value::Function(Function::NativeFunction(function)) => {
                    Ok(Value::Function(Function::NativeMethod(NativeMethod {
                        func: function,
                        inst_value: Box::new(Value::NativeObjet(self)),
                    })))
                }
                v => Ok(v),
            },
        }
    }

    fn do_op(
        self: Arc<Self>,
        vm: &mut VM,
        op: crate::compiler::bytecode::BinOpcode,
        other: Value,
    ) -> Result<Value, RuntimeError> {
        match op {
            BinOpcode::Div => {
                unpack_native!(chemin: &ASPath = other => {
                    chemin.0.clone()
                } else {
                    PathBuf::from(other.as_texte()?)
                });

                Ok(Value::native_objet(Self(self.0.join(chemin))))
            }
            _ => Err(RuntimeError::generic_err(format!(
                "Cet objet ne supporte pas l'opération {:?}",
                op
            ))),
        }
    }

    fn display(&self) -> String {
        self.0.display().to_string()
    }

    fn as_any(self: Arc<Self>) -> Arc<dyn Any> {
        self
    }
}

as_module! {
    module Chemin {}

    fn load(&self) {
        [
            as_module_fonction! {
                créer(chemin: {Texte | "Chemin.Chemin"}) {
                    unpack_native!(chemin: &ASPath = chemin => {
                        chemin.0.clone()
                    } else {
                        PathBuf::from(chemin.as_texte()?)
                    });

                    Ok(Value::native_objet(ASPath(chemin)))
                }
            },
            as_module_fonction! {
                parent(inst: {Texte | "Chemin.Chemin"}) {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(path
                            .parent()
                            .map(|p| Value::native_objet(ASPath(p.to_path_buf())))
                            .unwrap_or(Value::Nul))
                }
            },
            as_module_fonction! {
                nom(inst: {Texte | "Chemin.Chemin"}) {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(path
                            .file_name()
                            .map(|p| Value::native_objet(ASPath(PathBuf::from(p))))
                            .unwrap_or(Value::Nul))
                }
            },
            as_module_fonction! {
                extension(inst: {Texte | "Chemin.Chemin"}) {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(path
                            .extension()
                            .map(|p| Value::native_objet(ASPath(PathBuf::from(p))))
                            .unwrap_or(Value::Nul))
                }
            },
            as_module_fonction! {
                "La tige du fichier est le nom sans la dernière extenstion (ex: abc.tar.gz -> abc.tar)";
                tigeDuFichier(inst: {Texte | "Chemin.Chemin"}) {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(path
                            .file_stem()
                            .map(|p| Value::native_objet(ASPath(PathBuf::from(p))))
                            .unwrap_or(Value::Nul))
                }
            },
            as_module_fonction! {
                joindre(
                    inst: {Texte | "Chemin.Chemin"},
                    chemin: {Texte | "Chemin.Chemin"},
                ) {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });
                    unpack_native!(chemin: &ASPath = chemin => {
                        chemin.0.clone()
                    } else {
                        PathBuf::from(chemin.as_texte()?)
                    });

                    Ok(Value::native_objet(ASPath(path.join(chemin))))
                }
            },
            as_module_fonction! {
                avecNomFichier(
                    inst: {Texte | "Chemin.Chemin"},
                    filename: {Texte | "Chemin.Chemin"},
                ) {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });
                    unpack_native!(name: &ASPath = filename => {
                        name.0.clone()
                    } else {
                        PathBuf::from(filename.as_texte()?)
                    });

                    Ok(Value::native_objet(ASPath(path.with_file_name(name))))
                }
            },
            as_module_fonction! {
                avecExtension(
                    inst: {Texte | "Chemin.Chemin"},
                    extension: {Texte | "Chemin.Chemin"},
                ) {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });
                    unpack_native!(extension: &ASPath = extension => {
                        extension.0.clone()
                    } else {
                        PathBuf::from(extension.as_texte()?)
                    });

                    Ok(Value::native_objet(ASPath(path.with_extension(extension))))
                }
            },
            as_module_fonction! {
                canoniser(inst: {Texte | "Chemin.Chemin"}) {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(path
                            .canonicalize()
                            .map(|p| Value::native_objet(ASPath(PathBuf::from(p))))
                            .unwrap_or(Value::Nul))
                }
            },
            as_module_fonction! {
                existe(inst: {Texte | "Chemin.Chemin"}) {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(Value::Booleen(path.exists()))
                }
            },
            as_module_fonction! {
                estFichier(inst: {Texte | "Chemin.Chemin"}) {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(Value::Booleen(path.is_file()))
                }
            },
            as_module_fonction! {
                estDossier(inst: {Texte | "Chemin.Chemin"}) {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(Value::Booleen(path.is_dir()))
                }
            },
            as_module_fonction! {
                estRelatif(inst: {Texte | "Chemin.Chemin"}) {
                    let path = match inst {
                        Value::Texte(t) => PathBuf::from(t),
                        c @ Value::NativeObjet(..) => {
                            unpack_native!(c: &ASPath = c);
                            c.0.clone()
                        }
                        _ => unreachable!()
                    };

                    Ok(Value::Booleen(path.is_relative()))
                }
            },
            as_module_fonction! {
                estAbsolu(inst: {Texte | "Chemin.Chemin"}) {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(Value::Booleen(path.is_absolute()))
                }
            },
            as_module_fonction! {
                estLienSymbolique(inst: {Texte | "Chemin.Chemin"}) {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(Value::Booleen(path.is_symlink()))
                }
            },
            as_module_fonction! {
                estLienSymbolique(inst: {Texte | "Chemin.Chemin"}) {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(Value::Booleen(path.is_symlink()))
                }
            },
        ]
    }
}
