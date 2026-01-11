use std::{
    any::Any,
    collections::VecDeque,
    fs,
    io::{self, BufRead, BufReader, Read, Write},
    ops::Deref,
    path::PathBuf,
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
    stdlib::{path::ASPath, LazyModule},
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
                créer(
                    cmd: {Texte | "Chemin.Chemin"},
                    args: {Liste(Tout)} => Value::liste(vec![]),
                    dir: {Optionnel(Texte | "Chemin.Chemin")} => Value::Nul
                ) {
                    unpack_native!(cmd: &ASPath = cmd => {
                        cmd.0.clone()
                    } else {
                        PathBuf::from(cmd.as_texte()?)
                    });
                    unpack_native!(dir: &ASPath = dir => {
                        Ok(dir.0.clone())
                    } else {
                        dir.as_texte().map(|d| PathBuf::from(d))
                    });

                    let mut command = Command::new(cmd);
                    if let Ok(dir) = dir {
                        command.current_dir(fs::canonicalize(dir).unwrap());
                    }
                    command.args(args.as_liste()?
                            .read()
                            .unwrap()
                            .iter()
                            .map(|v| v.to_string()),
                    );

                    Ok(Value::NativeObjet(Arc::new(ProcessHandle {cmd: RwLock::new(command)})))
                }
            },
            as_module_fonction! {
                execAvecSortie(inst: {"Processus.SousProcessus"}) {
                    unpack_native!(cmd: &ProcessHandle = inst);
                    let output = cmd.cmd.write()
                        .unwrap()
                        .output()
                        .map_err(|e| RuntimeError::generic_err(format!("erreur lors de l'exécution de la commande:\n{}", e)))?;

                    Ok(Value::liste(vec![
                        Value::Texte(String::from_utf8(output.stdout).map_err(|e| RuntimeError::generic_err(e))?),
                        Value::Texte(String::from_utf8(output.stderr).map_err(|e| RuntimeError::generic_err(e))?),
                        output.status.code().map(|code| Value::Entier(code as i64)).unwrap_or(Value::Nul),
                    ]))
                }
            },
            as_module_fonction! {
                exec(
                    inst: {"Processus.SousProcessus"},
                    opt: {Dict(Tout)} => Value::dict_from_iter(vec![("texte", Value::Booleen(true))])
                ) {
                    unpack_native!(cmd: &ProcessHandle = inst);

                    let child = cmd.cmd.write()
                        .unwrap()
                        .spawn()
                        .map_err(|e| RuntimeError::generic_err(format!("erreur lors de l'exécution de la commande:\n{}", e)))?;

                    let opts = &opt.as_dict()?.read().unwrap().members.get("texte").unwrap_or(&Value::Booleen(true)).as_bool()?;

                    Ok(Value::NativeObjet(Arc::new(ChildProcessHandle {child: RwLock::new(child), text: *opts })))
                }
            },
            as_module_fonction! {
                obtenirSortie(inst: {"Processus.ProcessusEnfant"}) {
                    unpack_native!(subprocess: &ChildProcessHandle = inst);
                    let mut stdout = subprocess.child.write()
                        .unwrap()
                        .stdout
                        .take()
                        .ok_or_else(|| RuntimeError::generic_err(format!("erreur lors de l'exécution de la commande:\n")))?;

                    if subprocess.text {
                        let mut output = String::new();
                        _ = stdout.read_to_string(&mut output).map_err(|e| RuntimeError::generic_err(format!("erreur lors de l'exécution de la commande:\n{}", e)))?;
                        Ok(Value::Texte(output))
                    } else {
                        let mut output = Vec::new();
                        _ = stdout.read_to_end(&mut output).map_err(|e| RuntimeError::generic_err(format!("erreur lors de l'exécution de la commande:\n{}", e)))?;
                        Ok(Value::liste(output.into_iter().map(|v| Value::Entier(v as i64)).collect()))
                    }

                }
            },
        ]
    }
}
