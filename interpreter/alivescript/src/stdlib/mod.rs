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

mod macros;

pub mod builtins;
mod datetime;
mod env;
mod io;
mod liste;
mod module;
mod os;
mod path;
mod projet;
mod subprocess;
mod sys;
mod texte;
mod dict;
mod math;
mod test;
mod random;
mod debug;
mod toml;

pub fn get_stdlib() -> HashMap<String, Arc<dyn LazyModule>> {
    let mut stdlib: Vec<Arc<dyn LazyModule>> = Vec::new();

    stdlib.push(Arc::new(texte::Texte {}));
    stdlib.push(Arc::new(liste::Liste {}));
    stdlib.push(Arc::new(dict::Dict {}));
    stdlib.push(Arc::new(test::Test {}));
    stdlib.push(Arc::new(sys::Systeme {}));
    stdlib.push(Arc::new(random::Aleatoire {}));
    stdlib.push(Arc::new(debug::Debug {}));
    stdlib.push(Arc::new(io::ES {}));
    stdlib.push(Arc::new(os::SE {}));
    stdlib.push(Arc::new(env::Env {}));
    stdlib.push(Arc::new(module::Module {}));
    stdlib.push(Arc::new(subprocess::Processus {}));
    stdlib.push(Arc::new(projet::Projet {}));
    stdlib.push(Arc::new(path::Chemin {}));
    stdlib.push(Arc::new(math::Math {}));
    stdlib.push(Arc::new(toml::Toml {}));

    HashMap::from_iter(
        stdlib
            .into_iter()
            .map(|lz_mod| (lz_mod.name().to_string(), lz_mod)),
    )
}

pub trait LazyModule {
    fn name(&self) -> &'static str;
    fn load(&self) -> ArcModule;
}
