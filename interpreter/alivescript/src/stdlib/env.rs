use std::{
    any::Any,
    env, fs,
    io::{self, BufRead, BufReader, Read, Write},
    ops::Deref,
    sync::{Arc, RwLock},
};

use crate::{
    as_module, as_module_fonction,
    compiler::{
        obj::{Function, Value},
        value::{ArcNativeObjet, NativeMethod, NativeObjet, Type},
    },
    runtime::err::RuntimeError,
    stdlib::LazyModule,
    unpack,
};

as_module! {
    module Env {}

    fn load(&self) {
        [
            as_module_fonction! {
                execActuel(): Type::Texte => {
                    Ok(Some(Value::Texte(
                        env::current_exe()
                            .map(|p| p.display().to_string())
                            .unwrap_or(String::new())
                    )))
                }
            },
            as_module_fonction! {
                dossierActuel(): Type::Texte => {
                    Ok(Some(Value::Texte(
                        env::current_dir()
                            .map(|p| p.display().to_string())
                            .unwrap_or(String::new())
                    )))
                }
            },
            as_module_fonction! {
                var(name: Type::Texte): Type::Texte => {
                    Ok(Some(
                        env::var(name.to_string())
                            .map(Value::Texte)
                            .unwrap_or(Value::Nul)
                    ))
                }
            },
            as_module_fonction! {
                vars(): Type::Liste => {
                    Ok(Some(Value::liste(
                        env::vars()
                            .map(|(k, v)| Value::liste(vec![Value::Texte(k), Value::Texte(v)]))
                            .collect()
                    )))
                }
            },
        ]
    }
}
