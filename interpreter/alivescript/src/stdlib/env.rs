use std::{
    any::Any,
    env, fs,
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
    stdlib::{path::ASPath, LazyModule},
    unpack,
};

as_module! {
    module Env {}

    fn load(&self) {
        [
            as_module_fonction! {
                fichierActuel[vm]() {
                    Ok(Value::native_objet(ASPath(PathBuf::from(vm.file()))))
                }
            },
            as_module_fonction! {
                cheminExec() {
                    Ok(Value::native_objet(
                        env::current_exe()
                            .map(|p| ASPath(p))
                            .map_err(|e| RuntimeError::generic_err("Impossible d'obtenir le chemin de l'exécutable."))?
                    ))
                }
            },
            as_module_fonction! {
                dossierDeTravail() {
                    Ok(Value::native_objet(
                        env::current_dir()
                            .map(|p| ASPath(p))
                            .map_err(|e| RuntimeError::generic_err("Impossible d'obtenir le dossier de travail."))?
                    ))
                }
            },
            as_module_fonction! {
                args(): Type::Liste => {
                    Ok(Value::liste(
                        std::env::args()
                            .map(|arg| Value::Texte(arg))
                            .collect()
                    ))
                }
            },
            as_module_fonction! {
                var(name: {Texte}): Type::Texte => {
                    Ok(
                        env::var(name.to_string())
                            .map(Value::Texte)
                            .unwrap_or(Value::Nul)
                    )
                }
            },
            as_module_fonction! {
                vars(): Type::Liste => {
                    Ok(Value::liste(
                        env::vars()
                            .map(|(k, v)| Value::liste(vec![Value::Texte(k), Value::Texte(v)]))
                            .collect()
                    ))
                }
            },
            as_module_fonction! {
                changerVar(name: {Texte}, val: {Texte}): Type::Texte => {
                    unsafe {
                        env::set_var(name.to_string(), val.to_string());
                    }
                    Ok(Value::Nul)
                }
            },
        ]
    }
}
