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
    module Aleatoire as "Aléatoire" {}

    fn load(&self) {
        [
            as_module_fonction! {
                choix(iter: {iterable()}): Type::Tout => {
                    match iter {
                        Value::Liste(lst) => {
                            let i = random_range(0..lst.read().unwrap().len());
                            Ok(lst.read().unwrap()[i].clone())
                        }
                        _ => Err(RuntimeError::value_error(
                            format!("dans la fonction 'Alétoire.choix', argument #1 invalide '{}'", iter.get_type())
                        ))
                    }
                }
            },
            as_module_fonction! {
                entre(min: {Entier}, max: {Entier}): Type::Entier => {
                    let min = min.as_entier().unwrap();
                    let max = max.as_entier().unwrap();
                    let i = random_range(min..max);
                    Ok(Value::Entier(i))
                }
            },
        ]
    }
}

