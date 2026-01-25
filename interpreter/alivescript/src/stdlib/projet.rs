use core::time;
use std::{
    any::Any,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    str::FromStr,
    sync::{Arc, LazyLock, RwLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use dyn_fmt::AsStrFormatExt;
use rand::random_range;
use uuid::timestamp;

use crate::{
    as_module, as_module_fonction,
    compiler::{
        obj::Function,
        value::{NativeMethod, NativeObjet},
    },
    runtime::{
        config::{PermissionSet, VMAction, VMConfig},
        vm::VM,
    },
    unpack, unpack_native,
};
use crate::{
    compiler::{
        obj::Value,
        value::{ASModule, ArcModule, Type},
    },
    runtime::err::RuntimeError,
};

as_module! {
    module Projet {}

    fn load(&self) {
        [
            as_module_fonction! {
                configurer[vm](config: {Dict(Tout)}) {
                    vm.insert_exported_global_value(("__AS_PROJET", config));
                    Ok(Value::Nul)
                }
            },
            as_module_fonction! {
                ajouterDépendance[vm](dep_config: {Dict(Tout)}) {
                    let proj = vm.get_global("__AS_PROJET")
                        .ok_or_else(|| RuntimeError::generic_err(
                            format!("Le projet doit être configuré avant de pouvoir ajouter des dépendances")
                        ))?;

                    let proj = proj.value.as_dict()?;

                    proj.write().unwrap().members
                        .entry(String::from("dépendances"))
                        .and_modify(|l| l.as_liste().unwrap().write().unwrap().push(dep_config.clone()))
                        .or_insert_with(|| Value::liste(vec![dep_config]));

                    Ok(Value::Nul)
                }
            },
        ]
    }
}
