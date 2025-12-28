use std::{
    any::Any,
    fs,
    io::{self, BufRead, BufReader, Read, Write},
    ops::Deref,
    sync::{Arc, RwLock},
};

use crate::{
    as_module, as_module_fonction,
    compiler::{
        obj::{Function, Value},
        value::{ArcNativeObjet, NativeMethod, NativeObjet, Type},
    },
    runtime::err::RuntimeError,
    stdlib::LazyModule,
    unpack,
};

as_module! {
    module SE {}

    fn load(&self) {
        [
            as_module_fonction! {
                fichierActuel[vm](): Type::Texte => {
                    Ok(Some(Value::Texte(vm.file().to_string())))
                }
            },
        ]
    }
}
