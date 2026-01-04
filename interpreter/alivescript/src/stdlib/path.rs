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
        let es = vm.get_std_module("Chemin");
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
                créer(chemin: Type::Texte) => {
                    Ok(Some(Value::native_objet(ASPath(PathBuf::from(chemin.as_texte()?)))))
                }
            },
            as_module_fonction! {
                parent(inst: Type::union_of(Type::Texte, Type::objet("Chemin.Chemin"))) => {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(Some(path
                            .parent()
                            .map(|p| Value::native_objet(ASPath(p.to_path_buf())))
                            .unwrap_or(Value::Nul)))
                }
            },
            as_module_fonction! {
                nom(inst: Type::union_of(Type::Texte, Type::objet("Chemin.Chemin"))) => {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(Some(path
                            .file_name()
                            .map(|p| Value::native_objet(ASPath(PathBuf::from(p))))
                            .unwrap_or(Value::Nul)))
                }
            },
            as_module_fonction! {
                extension(inst: Type::union_of(Type::Texte, Type::objet("Chemin.Chemin"))) => {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(Some(path
                            .extension()
                            .map(|p| Value::native_objet(ASPath(PathBuf::from(p))))
                            .unwrap_or(Value::Nul)))
                }
            },
            as_module_fonction! {
                "La tige du fichier est le nom sans la dernière extenstion (ex: abc.tar.gz -> abc.tar)";
                tigeDuFichier(inst: Type::union_of(Type::Texte, Type::objet("Chemin.Chemin"))) => {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(Some(path
                            .file_stem()
                            .map(|p| Value::native_objet(ASPath(PathBuf::from(p))))
                            .unwrap_or(Value::Nul)))
                }
            },
            as_module_fonction! {
                joindre(
                    inst: Type::union_of(Type::Texte, Type::objet("Chemin.Chemin")),
                    chemin: Type::union_of(Type::Texte, Type::objet("Chemin.Chemin")),
                ) => {
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

                    Ok(Some(Value::native_objet(ASPath(path.join(chemin)))))
                }
            },
            as_module_fonction! {
                avecNomFichier(
                    inst: Type::union_of(Type::Texte, Type::objet("Chemin.Chemin")),
                    filename: Type::union_of(Type::Texte, Type::objet("Chemin.Chemin")),
                ) => {
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

                    Ok(Some(Value::native_objet(ASPath(path.with_file_name(name)))))
                }
            },
            as_module_fonction! {
                avecExtension(
                    inst: Type::union_of(Type::Texte, Type::objet("Chemin.Chemin")),
                    extension: Type::union_of(Type::Texte, Type::objet("Chemin.Chemin")),
                ) => {
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

                    Ok(Some(Value::native_objet(ASPath(path.with_extension(extension)))))
                }
            },
            as_module_fonction! {
                canoniser(inst: Type::union_of(Type::Texte, Type::objet("Chemin.Chemin"))) => {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(Some(path
                            .canonicalize()
                            .map(|p| Value::native_objet(ASPath(PathBuf::from(p))))
                            .unwrap_or(Value::Nul)))
                }
            },
            as_module_fonction! {
                existe(inst: Type::union_of(Type::Texte, Type::objet("Chemin.Chemin"))) => {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(Some(Value::Booleen(path.exists())))
                }
            },
            as_module_fonction! {
                estFichier(inst: Type::union_of(Type::Texte, Type::objet("Chemin.Chemin"))) => {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(Some(Value::Booleen(path.is_file())))
                }
            },
            as_module_fonction! {
                estDossier(inst: Type::union_of(Type::Texte, Type::objet("Chemin.Chemin"))) => {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(Some(Value::Booleen(path.is_dir())))
                }
            },
            as_module_fonction! {
                estRelatif(inst: Type::union_of(Type::Texte, Type::objet("Chemin.Chemin"))) => {
                    let path = match inst {
                        Value::Texte(t) => PathBuf::from(t),
                        c @ Value::NativeObjet(..) => {
                            unpack_native!(c: &ASPath = c);
                            c.0.clone()
                        }
                        _ => unreachable!()
                    };

                    Ok(Some(Value::Booleen(path.is_relative())))
                }
            },
            as_module_fonction! {
                estAbsolu(inst: Type::union_of(Type::Texte, Type::objet("Chemin.Chemin"))) => {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(Some(Value::Booleen(path.is_absolute())))
                }
            },
            as_module_fonction! {
                estLienSymbolique(inst: Type::union_of(Type::Texte, Type::objet("Chemin.Chemin"))) => {
                    unpack_native!(path: &ASPath = inst => {
                        path.0.clone()
                    } else {
                        PathBuf::from(inst.as_texte()?)
                    });

                    Ok(Some(Value::Booleen(path.is_symlink())))
                }
            },
        ]
    }
}
