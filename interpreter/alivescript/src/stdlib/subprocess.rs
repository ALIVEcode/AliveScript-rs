use std::{
    any::Any,
    collections::VecDeque,
    fs,
    io::{self, BufRead, BufReader, Read, Write},
    ops::Deref,
    process::{Child, Command},
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
    unpack, unpack_native,
};

#[derive(Debug)]
struct ProcessHandle {
    cmd: RwLock<Command>,
}

impl NativeObjet for ProcessHandle {
    fn type_name(&self) -> &'static str {
        "Processus.SousProcessus"
    }

    fn get_member(
        self: Arc<Self>,
        vm: &mut crate::runtime::vm::VM,
        name: &str,
    ) -> Result<Value, crate::runtime::err::RuntimeError> {
        let es = vm.get_std_module("Processus");
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

#[derive(Debug)]
struct ChildProcessHandle {
    child: RwLock<Child>,
    text: bool,
}

impl NativeObjet for ChildProcessHandle {
    fn type_name(&self) -> &'static str {
        "Processus.ProcessusEnfant"
    }

    fn get_member(
        self: Arc<Self>,
        vm: &mut crate::runtime::vm::VM,
        name: &str,
    ) -> Result<Value, crate::runtime::err::RuntimeError> {
        let es = vm.get_std_module("Processus");
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
    module Processus {}

    fn load(&self) {
        [
            as_module_fonction! {
                créer(cmd: Type::Texte, args: Type::liste(Type::Texte) => Value::liste(vec![])) => {
                    let mut command = Command::new(cmd.as_texte()?);
                    command.args(args.as_liste()?
                            .read()
                            .unwrap()
                            .iter()
                            .map(|v| v.to_string()),
                    );

                    Ok(Some(Value::NativeObjet(Arc::new(ProcessHandle {cmd: RwLock::new(command)}))))
                }
            },
            as_module_fonction! {
                execAvecSortie(inst: Type::Objet(String::from("Processus.SousProcessus"))) => {
                    unpack_native!(cmd: &ProcessHandle = inst);
                    let output = cmd.cmd.write()
                        .unwrap()
                        .output()
                        .map_err(|e| RuntimeError::generic_err(format!("erreur lors de l'exécution de la commande:\n{}", e)))?;

                    Ok(Some(Value::liste(vec![
                        Value::Texte(String::from_utf8(output.stdout).map_err(|e| RuntimeError::generic_err(e))?),
                        Value::Texte(String::from_utf8(output.stderr).map_err(|e| RuntimeError::generic_err(e))?),
                        output.status.code().map(|code| Value::Entier(code as i64)).unwrap_or(Value::Nul),
                    ])))
                }
            },
            as_module_fonction! {
                exec(
                    inst: Type::Objet(String::from("Processus.SousProcessus")),
                    opt: Type::dict_val_tout() => Value::dict(vec![("texte", Value::Booleen(true))])
                ) => {
                    unpack_native!(cmd: &ProcessHandle = inst);

                    let child = cmd.cmd.write()
                        .unwrap()
                        .spawn()
                        .map_err(|e| RuntimeError::generic_err(format!("erreur lors de l'exécution de la commande:\n{}", e)))?;

                    let opts = &opt.as_dict()?.read().unwrap().members.get("texte").unwrap_or(&Value::Booleen(true)).as_bool()?;

                    Ok(Some(Value::NativeObjet(Arc::new(ChildProcessHandle {child: RwLock::new(child), text: *opts }))))
                }
            },
            as_module_fonction! {
                obtenirSortie(inst: Type::Objet(String::from("Processus.ProcessusEnfant"))) => {
                    unpack_native!(subprocess: &ChildProcessHandle = inst);
                    let mut stdout = subprocess.child.write()
                        .unwrap()
                        .stdout
                        .take()
                        .ok_or_else(|| RuntimeError::generic_err(format!("erreur lors de l'exécution de la commande:\n")))?;

                    if subprocess.text {
                        let mut output = String::new();
                        _ = stdout.read_to_string(&mut output).map_err(|e| RuntimeError::generic_err(format!("erreur lors de l'exécution de la commande:\n{}", e)))?;
                        Ok(Some(Value::Texte(output)))
                    } else {
                        let mut output = Vec::new();
                        _ = stdout.read_to_end(&mut output).map_err(|e| RuntimeError::generic_err(format!("erreur lors de l'exécution de la commande:\n{}", e)))?;
                        Ok(Some(Value::liste(output.into_iter().map(|v| Value::Entier(v as i64)).collect())))
                    }

                }
            },
        ]
    }
}
