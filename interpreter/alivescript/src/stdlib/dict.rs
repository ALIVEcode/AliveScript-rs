use core::time;
use std::{
    collections::HashMap,
    marker::PhantomData,
    process::exit,
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
    module Dict {}

    fn load(&self) {
        [
            as_module_fonction! {
                taille(inst: {Dict(Tout)}): Type::Entier => {
                    unpack!(Value::Dict(d) = inst);

                    Ok(Value::Entier(d.read().unwrap().members.len() as i64))
                }
            },
            as_module_fonction! {
                clés(inst: {Dict(Tout)}) {
                    unpack!(Value::Dict(d) = inst);

                    Ok(Value::liste(d.read().unwrap().members.keys().map(|k| Value::Texte(k.clone())).collect()))
                }
            },
            as_module_fonction! {
                valeurs(inst: {Dict(Tout)}) {
                    unpack!(Value::Dict(d) = inst);

                    Ok(Value::liste(d.read().unwrap().members.values().map(|v| v.clone()).collect()))
                }
            },
            as_module_fonction! {
                entrées(inst: {Dict(Tout)}) {
                    unpack!(Value::Dict(d) = inst);

                    Ok(Value::liste(d.read().unwrap().members
                        .iter()
                        .map(|(k, v)|
                            Value::liste(vec![Value::Texte(k.clone()), v.clone()])).collect()))
                }
            },
            as_module_fonction! {
                obtenir(inst: {Dict(Tout)}, cle: {Texte}, valeur: {Optionnel(Tout)} => Value::Nul) {
                    unpack!(Value::Dict(d) = inst);
                    let cle = cle.as_texte()?;

                    Ok(d.read().unwrap().get(cle).cloned().unwrap_or(valeur))
                }
            },
            as_module_fonction! {
                contient(inst: {Dict(Tout)}, cle: {Texte}) {
                    unpack!(Value::Dict(d) = inst);
                    let cle = cle.as_texte()?;

                    Ok(Value::Booleen(d.read().unwrap().members.contains_key(cle)))
                }
            },
        ]
    }
}
