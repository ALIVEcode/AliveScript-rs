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

#[derive(Debug)]
struct ModuleBuilder {
    path: String,
    module_searcher: RwLock<Option<Function>>,
    vm_config: RwLock<VMConfig>,
}

impl NativeObjet for ModuleBuilder {
    fn type_name(&self) -> &'static str {
        "Module.Constructeur"
    }

    fn get_member(
        self: Arc<Self>,
        vm: &mut crate::runtime::vm::VM,
        name: &str,
    ) -> Result<Value, crate::runtime::err::RuntimeError> {
        let es = vm.get_std_module("Module");
        match es.read().unwrap().get_member(name)? {
            Value::Function(Function::NativeFunction(function)) => {
                Ok(Value::Function(Function::NativeMethod(NativeMethod {
                    func: function,
                    inst_value: Box::new(Value::NativeObjet(self)),
                })))
            }
            v => Ok(v),
        }
    }

    fn as_any(self: Arc<Self>) -> Arc<dyn Any> {
        self
    }
}

as_module! {
    module Projet {}

    fn load(&self) {
        [
            as_module_fonction! {
                configurer[vm](config: Type::dict_val_tout()) => {
                    vm.insert_exported_global(("__AS_PROJET", config));
                    Ok(Some(Value::Nul))
                }
            },
            as_module_fonction! {
                ajouterDépendance[vm](dep_config: Type::dict_val_tout()) => {
                    let proj = vm.get_global("__AS_PROJET")
                        .ok_or_else(|| RuntimeError::generic_err(
                            format!("Le projet doit être configuré avant de pouvoir ajouter des dépendances")
                        ))?;

                    let proj = proj.as_dict()?;

                    proj.write().unwrap().members
                        .entry(String::from("dépendances"))
                        .and_modify(|l| l.as_liste().unwrap().write().unwrap().push(dep_config.clone()))
                        .or_insert_with(|| Value::liste(vec![dep_config]));

                    Ok(Some(Value::Nul))
                }
            },
        ]
    }
}
