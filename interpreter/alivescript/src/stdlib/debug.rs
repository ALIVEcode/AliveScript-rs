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
    module Debug as "Débug" {}

    fn load(&self) {
        [
            as_module_fonction! {
                nb(inst: {Texte}): Type::Booleen => {
                    let inst = inst.as_texte().unwrap();
                    Ok(Value::Booleen(inst.chars().all(|c| c.is_ascii_digit())))
                }
            },
        ]
    }
}

