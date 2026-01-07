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
                taille(inst: {Texte}) {
                    let inst = inst.as_texte().unwrap();
                    Ok(Value::Entier(inst.len() as i64))
                }
            },
            as_module_fonction! {
                estNumérique(inst: {Texte}) {
                    let inst = inst.as_texte().unwrap();
                    Ok(Value::Booleen(inst.chars().all(|c| c.is_ascii_digit())))
                }
            },
            as_module_fonction! {
                format(inst: {Texte}, args: {Liste(Tout)}) {
                    let inst = inst.as_texte().unwrap();
                    let args = args.as_liste().unwrap();
                    Ok(Value::Texte(inst.format(args.read().unwrap().iter())))
                }
            },
            as_module_fonction! {
                commencePar(inst: {Texte}, prefix: {Texte}) {
                    let inst = inst.as_texte().unwrap();
                    let prefix = prefix.as_texte().unwrap();
                    Ok(Value::Booleen(inst.starts_with(prefix)))
                }
            },
            as_module_fonction! {
                finiPar(inst: {Texte}, prefix: {Texte}) {
                    let inst = inst.as_texte().unwrap();
                    let prefix = prefix.as_texte().unwrap();
                    Ok(Value::Booleen(inst.ends_with(prefix)))
                }
            },
            as_module_fonction! {
                diviser(inst: {Texte}, pat: {Texte}) {
                    let inst = inst.as_texte().unwrap();
                    let pat = pat.as_texte().unwrap();
                    Ok(Value::liste(
                        inst.split(pat)
                            .map(|s| Value::Texte(s.to_string()))
                            .collect()
                    ))
                }
            },
            as_module_fonction! {
                raser(inst: {Texte}, pat: {Optionnel(Texte)} => Value::Nul) {
                    let inst = inst.as_texte().unwrap();
                    let pat = pat.as_texte();
                    let val = match pat {
                        Ok(pat) => inst.trim_start_matches(pat).trim_end_matches(pat).to_string(),
                        Err(_) => inst.trim().to_string(),
                    };

                    Ok(Value::Texte(val))
                }
            },
            as_module_fonction! {
                raserDébut(inst: {Texte}, pat: {Optionnel(Texte)}) {
                    let inst = inst.as_texte().unwrap();
                    let pat = pat.as_texte();
                    let val = match pat {
                        Ok(pat) => inst.trim_start_matches(pat).to_string(),
                        Err(_) => inst.trim_start().to_string(),
                    };

                    Ok(Value::Texte(val))
                }
            },
            as_module_fonction! {
                raserFin(inst: {Texte}, pat: {Optionnel(Texte)} => Value::Nul) {
                    let inst = inst.as_texte().unwrap();
                    let pat = pat.as_texte();
                    let val = match pat {
                        Ok(pat) => inst.trim_end_matches(pat).to_string(),
                        Err(_) => inst.trim_end().to_string(),
                    };

                    Ok(Value::Texte(val))
                }
            },
            as_module_fonction! {
                sansPréfix(inst: {Texte}, pat: {Optionnel(Texte)}) {
                    let inst = inst.as_texte()?;
                    let pat = pat.as_texte()?;

                    Ok(Value::Texte(inst.strip_prefix(pat).unwrap_or(inst).to_string()))
                }
            },
            as_module_fonction! {
                sansSuffix(inst: {Texte}, pat: {Texte}) {
                    let inst = inst.as_texte()?;
                    let pat = pat.as_texte()?;

                    Ok(Value::Texte(inst.strip_suffix(pat).unwrap_or(inst).to_string()))
                }
            },
        ]
    }
}
