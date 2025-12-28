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

use crate::{as_module, as_module_fonction, runtime::vm::VM, unpack};
use crate::{
    compiler::{
        obj::Value,
        value::{ASModule, ArcModule, Type},
    },
    runtime::err::RuntimeError,
};

as_module! {
    module Module {}

    fn load(&self) {
        [
            as_module_fonction! {
                charger(chemin: Type::Texte): Type::Module => {
                    let mut vm = VM::new(String::new());
                    let module = vm.run_file_to_module(chemin.as_texte().unwrap())?;
                    Ok(Some(Value::Module(module)))
                }
            },
        ]
    }
}
