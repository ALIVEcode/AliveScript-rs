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
    module Module {}

    fn load(&self) {
        [
            as_module_fonction! {
                configurer(chemin: Type::Texte, obj: Type::dict_val_tout()) => {
                    unpack!(Value::Dict(d) = obj);

                    let mut vm = VM::new(String::new());
                    let mut builder = ModuleBuilder{
                        path: chemin.as_texte()?.to_string(),
                        vm_config: RwLock::new(vm.config().clone()),
                    };

                    let d = d.read().unwrap();
                    let included_modules = d.get("modulesPermis");
                    let excluded_modules = d.get("modulesInterdits");

                    if included_modules.as_ref().is_some_and(|_| excluded_modules.as_ref().is_some()){
                        return Err(RuntimeError::generic_err(
                            "impossible de spécifier à la fois les modules permis et interdits. Veuillez seulement spécifier un des deux."
                        ));
                    }

                    let included_perms = d.get("actionsPermises");
                    let excluded_perms = d.get("actionsInterdites");

                    if included_perms.as_ref().is_some_and(|_| excluded_perms.as_ref().is_some()){
                        return Err(RuntimeError::generic_err(
                            "impossible de spécifier à la fois les actions permises et interdites. Veuillez seulement spécifier un des deux."
                        ));
                    }

                    if let Some(included_modules) = included_modules {
                        let included_modules = included_modules.as_liste()?
                            .read()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_texte().map(ToString::to_string)).collect::<Result<HashSet<_>, _>>()?;
                        builder.vm_config.write().unwrap().allowed_modules = Some(PermissionSet::Include(included_modules))
                    } else if let Some(excluded_modules) = excluded_modules {
                        let excluded_modules = excluded_modules.as_liste()?
                            .read()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_texte().map(ToString::to_string)).collect::<Result<HashSet<_>, _>>()?;
                        builder.vm_config.write().unwrap().allowed_modules = Some(PermissionSet::Exclude(excluded_modules))
                    }

                    if let Some(included_perms) = included_perms {
                        let included_perms = included_perms.as_liste()?
                            .read()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_texte().and_then(|s| VMAction::from_str(s))).collect::<Result<HashSet<_>, _>>()?;
                        builder.vm_config.write().unwrap().permissions = Some(PermissionSet::Include(included_perms))
                    } else if let Some(excluded_perms) = excluded_perms {
                        let excluded_perms = excluded_perms.as_liste()?
                            .read()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_texte().and_then(|s| VMAction::from_str(s))).collect::<Result<HashSet<_>, _>>()?;
                        builder.vm_config.write().unwrap().permissions = Some(PermissionSet::Exclude(excluded_perms))
                    }

                    Ok(Some(Value::NativeObjet(Arc::new(builder))))
                }
            },
            as_module_fonction! {
                rechercheModule(inst: Type::Objet(String::from("Module.Constructeur")), f: Type::Fonction) => {
                    unpack_native!(builder: &ModuleBuilder = inst);

                    let f = f.as_fonc()?;
                    match f {
                        Function::ClosureInst(f) => {
                            if !f.upvalues.is_empty() {
                                return Err(RuntimeError::generic_err(
                                    "Dans 'rechercheModule': La fonction de recherche ne peut pas capturer de variables extérieure"
                                ))
                            }
                        }
                        _ => {}
                    }
                    builder.vm_config.write().unwrap().module_searcher = Some(f.clone());

                    Ok(Some(inst))
                }
            },
            as_module_fonction! {
                charger[current_vm](chemin: Type::union_of(Type::Texte, Type::Objet(String::from("Module.Constructeur")))): Type::Module => {
                    match chemin {
                        obj @ Value::NativeObjet(..) => {
                            unpack_native!(builder: &ModuleBuilder = obj);
                            let module = current_vm.run_file_to_module_with_config(&builder.path, builder.vm_config.read().unwrap().clone())?;
                            Ok(Some(Value::Module(module)))
                        }
                        Value::Texte(chemin) => {
                            let mut vm = VM::new(String::new());
                            let module = vm.run_file_to_module(&chemin)?;
                            Ok(Some(Value::Module(module)))
                        }
                        _ => unreachable!()
                    }
                }
            },
        ]
    }
}
