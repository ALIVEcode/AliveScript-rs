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
    module Texte {}

    fn load(&self) {
        [
            as_module_fonction! {
                taille(inst: Type::Texte): Type::Entier => {
                    let inst = inst.as_texte().unwrap();
                    Ok(Some(Value::Entier(inst.len() as i64)))
                }
            },
            as_module_fonction! {
                estNumérique(inst: Type::Texte): Type::Booleen => {
                    let inst = inst.as_texte().unwrap();
                    Ok(Some(Value::Booleen(inst.chars().all(|c| c.is_ascii_digit()))))
                }
            },
            as_module_fonction! {
                format(inst: Type::Texte, args: Type::liste_tout()): Type::Texte => {
                    let inst = inst.as_texte().unwrap();
                    let args = args.as_liste().unwrap();
                    Ok(Some(Value::Texte(inst.format(args.read().unwrap().iter()))))
                }
            },
            as_module_fonction! {
                commencePar(inst: Type::Texte, prefix: Type::Texte) => {
                    let inst = inst.as_texte().unwrap();
                    let prefix = prefix.as_texte().unwrap();
                    Ok(Some(Value::Booleen(inst.starts_with(prefix))))
                }
            },
            as_module_fonction! {
                finiPar(inst: Type::Texte, prefix: Type::Texte) => {
                    let inst = inst.as_texte().unwrap();
                    let prefix = prefix.as_texte().unwrap();
                    Ok(Some(Value::Booleen(inst.ends_with(prefix))))
                }
            },
            as_module_fonction! {
                diviser(inst: Type::Texte, pat: Type::Texte): Type::Liste => {
                    let inst = inst.as_texte().unwrap();
                    let pat = pat.as_texte().unwrap();
                    Ok(Some(Value::liste(
                        inst.split(pat)
                            .map(|s| Value::Texte(s.to_string()))
                            .collect()
                    )))
                }
            },
            as_module_fonction! {
                raser(inst: Type::Texte, pat: Type::optional(Type::Texte) => Value::Nul) => {
                    let inst = inst.as_texte().unwrap();
                    let pat = pat.as_texte();
                    let val = match pat {
                        Ok(pat) => inst.trim_start_matches(pat).trim_end_matches(pat).to_string(),
                        Err(_) => inst.trim().to_string(),
                    };

                    Ok(Some(Value::Texte(val)))
                }
            },
            as_module_fonction! {
                raserDébut(inst: Type::Texte, pat: Type::optional(Type::Texte)) => {
                    let inst = inst.as_texte().unwrap();
                    let pat = pat.as_texte();
                    let val = match pat {
                        Ok(pat) => inst.trim_start_matches(pat).to_string(),
                        Err(_) => inst.trim_start().to_string(),
                    };

                    Ok(Some(Value::Texte(val)))
                }
            },
            as_module_fonction! {
                raserFin(inst: Type::Texte, pat: Type::optional(Type::Texte) => Value::Nul) => {
                    let inst = inst.as_texte().unwrap();
                    let pat = pat.as_texte();
                    let val = match pat {
                        Ok(pat) => inst.trim_end_matches(pat).to_string(),
                        Err(_) => inst.trim_end().to_string(),
                    };

                    Ok(Some(Value::Texte(val)))
                }
            },
        ]
    }
}
