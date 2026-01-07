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

use crate::{as_module, as_module_fonction, as_module_var, unpack};
use crate::{
    compiler::{
        obj::Value,
        value::{ASModule, ArcModule, Type},
    },
    runtime::err::RuntimeError,
};

as_module! {
    module Math {}

    fn load(&self) {
        [
            as_module_var!(const PI: {Decimal} = Value::Decimal(std::f64::consts::PI)),
            as_module_var!(const E: {Decimal} = Value::Decimal(std::f64::consts::E)),
            as_module_var!(const TAU: {Decimal} = Value::Decimal(std::f64::consts::TAU)),
            as_module_var!(const INF: {Decimal} = Value::Decimal(f64::INFINITY)),
            as_module_var!(const INF_NÉG: {Decimal} = Value::Decimal(f64::NEG_INFINITY)),
            as_module_var!(const PUN: {Decimal} = Value::Decimal(f64::NAN)),
            as_module_var!(const NAN: {Decimal} = Value::Decimal(f64::NAN)),

            as_module_fonction! {
                plafond(x: {nombre()}) {
                    Ok(Value::Nul)
                }
            },
            as_module_fonction! {
                plancher(x: {nombre()}) {
                    Ok(Value::Nul)
                }
            },
            as_module_fonction! {
                tronquer(x: {nombre()}) {
                    Ok(Value::Nul)
                }
            },
            as_module_fonction! {
                arrondir(x: {nombre()}) {
                    Ok(Value::Nul)
                }
            },
            as_module_fonction! {
                abs(x: {nombre()}) {
                    Ok(Value::Nul)
                }
            },
            as_module_fonction! {
                racine(x: {nombre()}, n: {nombre()} => Value::Entier(2)) {
                    Ok(Value::Nul)
                }
            },
            as_module_fonction! {
                exp(x: {nombre()}, n: {Entier} => Value::Entier(2)) {
                    Ok(Value::Nul)
                }
            },
        ]
    }
}
