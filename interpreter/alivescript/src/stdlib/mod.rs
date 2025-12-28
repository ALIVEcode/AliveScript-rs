use core::time;
use std::{
    collections::HashMap,
    marker::PhantomData,
    sync::{Arc, LazyLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use dyn_fmt::AsStrFormatExt;
use rand::random_range;
use uuid::timestamp;

use crate::{as_module, as_module_fonction, unpack};
use crate::{
    compiler::{
        obj::Value,
        value::{ASModule, ArcModule, Type},
    },
    runtime::err::RuntimeError,
};

mod macros;

pub mod builtins;
mod env;
mod io;
mod os;
mod texte;
mod module;

as_module! {
    module Test { }

    fn load(&self) {
        [
            as_module_fonction! {
                affirmer(cond: Type::tout(), msg: Type::Texte): Type::Nul => {
                    if !cond.to_bool() {
                        let t = msg.as_texte().unwrap();
                        Err(RuntimeError::assertion_error(t))
                    } else {
                        Ok(None)
                    }
                }
            },
            as_module_fonction! {
                affirmerÉgaux(val1: Type::tout(), val2: Type::tout(), msg: Type::Texte): Type::Nul => {
                    if val1 != val2 {
                        let t = msg.as_texte().unwrap();
                        Err(RuntimeError::assertion_error(format!("{}\nGauche: {}\nDroite: {}", t, val1.repr(), val2.repr())))
                    } else {
                        Ok(None)
                    }
                }
            },
        ]
    }
}

as_module! {
    module Liste {}

    fn load(&self) {
        [
            as_module_fonction! {
                taille(inst: Type::liste_tout()): Type::Entier => {
                    unpack!(Value::Liste(lst) = inst);

                    Ok(Some(Value::Entier(lst.read().unwrap().len() as i64)))
                }
            },
            as_module_fonction! {
                ajouter(inst: Type::liste_tout(), val: Type::Tout): Type::Nul => {
                    unpack!(Value::Liste(lst) = inst);

                    lst.write().unwrap().push(val.clone());

                    Ok(Some(Value::Nul))
                }
            },
        ]
    }
}

as_module! {
    module Dict {}

    fn load(&self) {
        [
            as_module_fonction! {
                taille(inst: Type::dict_tout()): Type::Entier => {
                    unpack!(Value::Dict(d) = inst);

                    Ok(Some(Value::Entier(d.read().unwrap().members.len() as i64)))
                }
            },
        ]
    }
}

as_module! {
    module Aleatoire as "Aléatoire" {}

    fn load(&self) {
        [
            as_module_fonction! {
                choix(iter: Type::iterable()): Type::Tout => {
                    match iter {
                        Value::Liste(lst) => {
                            let i = random_range(0..lst.read().unwrap().len());
                            Ok(Some(lst.read().unwrap()[i].clone()))
                        }
                        _ => Err(RuntimeError::value_error(
                            format!("dans la fonction 'Alétoire.choix', argument #1 invalide '{}'", iter.get_type())
                        ))
                    }
                }
            },
            as_module_fonction! {
                entre(min: Type::Entier, max: Type::Entier): Type::Entier => {
                    let min = min.as_entier().unwrap();
                    let max = max.as_entier().unwrap();
                    let i = random_range(min..max);
                    Ok(Some(Value::Entier(i)))
                }
            },
        ]
    }
}

as_module! {
    module Systeme as "Système" {}

    fn load(&self) {
        [
            as_module_fonction! {
                temps(): Type::Entier => {
                    Ok(Some(Value::Entier(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as i64
                    )))
                }
            },
            as_module_fonction! {
                args(): Type::Liste => {
                    Ok(Some(Value::liste(
                        std::env::args()
                            .map(|arg| Value::Texte(arg))
                            .collect()
                    )))
                }
            },
        ]
    }
}

as_module! {
    module Debug as "Débug" {}

    fn load(&self) {
        [
            as_module_fonction! {
                nb(inst: Type::Texte): Type::Booleen => {
                    let inst = inst.as_texte().unwrap();
                    Ok(Some(Value::Booleen(inst.chars().all(|c| c.is_ascii_digit()))))
                }
            },
        ]
    }
}

pub fn get_stdlib() -> HashMap<String, Arc<dyn LazyModule>> {
    let mut stdlib: Vec<Arc<dyn LazyModule>> = Vec::new();

    stdlib.push(Arc::new(texte::Texte {}));
    stdlib.push(Arc::new(Liste {}));
    stdlib.push(Arc::new(Dict {}));
    stdlib.push(Arc::new(Test {}));
    stdlib.push(Arc::new(Systeme {}));
    stdlib.push(Arc::new(Aleatoire {}));
    stdlib.push(Arc::new(Debug {}));
    stdlib.push(Arc::new(io::ES {}));
    stdlib.push(Arc::new(os::SE {}));
    stdlib.push(Arc::new(env::Env {}));
    stdlib.push(Arc::new(module::Module {}));

    HashMap::from_iter(
        stdlib
            .into_iter()
            .map(|lz_mod| (lz_mod.name().to_string(), lz_mod)),
    )
}

pub trait LazyModule {
    fn name(&self) -> &'static str;
    fn load(&self) -> ArcModule;
}
