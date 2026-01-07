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

as_module! {
    module Liste {}

    fn load(&self) {
        [
            as_module_fonction! {
                taille(inst: {Liste(Tout)}): Type::Entier => {
                    unpack!(Value::Liste(lst) = inst);

                    Ok(Value::Entier(lst.read().unwrap().len() as i64))
                }
            },
            as_module_fonction! {
                ajouter(inst: {Liste(Tout)}, val: {Tout}): Type::Nul => {
                    unpack!(Value::Liste(lst) = inst);

                    lst.write().unwrap().push(val.clone());

                    Ok(Value::Nul)
                }
            },
            as_module_fonction! {
                joindre(inst: {Liste(Texte)}, sep: {Texte} => Value::Texte(String::from(" "))) {
                    unpack!(Value::Liste(lst) = inst);

                    Ok(Value::Texte(lst.read().unwrap()
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(sep.as_texte()?)))
                }
            },
            as_module_fonction! {
                map[vm](inst: {Liste(Tout)}, map: {Fonction}) {
                    unpack!(Value::Liste(lst) = inst);
                    unpack!(Value::Function(map) = map);

                    Ok(Value::liste(lst.read().unwrap()
                        .iter()
                        .map(|v| vm.run_fn(vec![v.clone()], &map))
                        .collect::<Result<_,_>>()?))
                }
            },
            as_module_fonction! {
                filtrer[vm](inst: {Liste(Tout)}, filtre: {Fonction}) {
                    unpack!(Value::Liste(lst) = inst);
                    unpack!(Value::Function(filtre) = filtre);

                    Ok(Value::liste(lst.read().unwrap()
                        .iter()
                        .filter_map(|v| {
                            let result = vm.run_fn(vec![v.clone()], &filtre);
                            match result {
                                Ok(r) if r.to_bool() => Some(Ok(v.clone())),
                                Ok(r) => None,
                                Err(e) => Some(Err(e))
                            }
                        })
                        .collect::<Result<_,_>>()?))
                }
            },
        ]
    }
}
