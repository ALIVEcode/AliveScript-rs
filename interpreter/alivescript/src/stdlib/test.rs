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
    module Test {}

    fn load(&self) {
        [
            as_module_fonction! {
                affirmer(cond: Type::tout(), msg: Type::Texte): Type::Nul => {
                    if !cond.to_bool() {
                        let t = msg.as_texte().unwrap();
                        Err(RuntimeError::assertion_error(t))
                    } else {
                        Ok(Value::Nul)
                    }
                }
            },
            as_module_fonction! {
                affirmerÉgaux(val1: Type::tout(), val2: Type::tout(), msg: Type::Texte): Type::Nul => {
                    if val1 != val2 {
                        let t = msg.as_texte().unwrap();
                        Err(RuntimeError::assertion_error(format!("{}\nGauche: {}\nDroite: {}", t, val1.repr(), val2.repr())))
                    } else {
                        Ok(Value::Nul)
                    }
                }
            },
        ]
    }
}
