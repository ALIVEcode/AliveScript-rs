use core::time;
use std::{
    any::Any,
    collections::HashMap,
    marker::PhantomData,
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
    runtime::vm::VM,
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
                créer(chemin: Type::Texte) => {
                    let mut vm = VM::new(String::new());
                    Ok(Some(Value::NativeObjet(Arc::new(ModuleBuilder{
                        path: chemin.as_texte()?.to_string(),
                        module_searcher: RwLock::new(None)
                    }))))
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
                    *builder.module_searcher.write().unwrap() = Some(f.clone());

                    Ok(Some(inst))
                }
            },
            as_module_fonction! {
                charger(chemin: Type::union_of(Type::Texte, Type::Objet(String::from("Module.Constructeur")))): Type::Module => {
                    match chemin {
                        obj @ Value::NativeObjet(..) => {
                            unpack_native!(builder: &ModuleBuilder = obj);
                            let mut vm = VM::new(String::new());
                            vm.set_module_searcher(builder.module_searcher.read().unwrap().clone());
                            let module = vm.run_file_to_module(&builder.path)?;
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
