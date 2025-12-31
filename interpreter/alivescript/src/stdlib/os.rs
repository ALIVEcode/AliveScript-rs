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
        ]
    }
}
