use std::{
    collections::{HashMap, VecDeque},
    env, fs, hint,
    io::{self, Write, stdin},
    path::{self, Path},
    sync::{Arc, LazyLock, RwLock},
    usize,
};

use crate::{
    compiler::{
        Compiler,
        bytecode::{BinCompcode, BinOpcode, JUMP_OFFSET, Opcode, instructions_to_string},
        obj::{ArcUpvalue, CallFrame, Function, Upvalue, UpvalueLocation, UpvalueSpec, Value},
        value::{
            ASDict, ASField, ASModule, ASObjet, ArcClosureInst, ArcClosureProto, ArcModule,
            ArcStructure, ClosureInst, ClosureProto, ModuleProto, NativeMethod,
        },
    },
    runtime::err::RuntimeError,
    stdlib::{LazyModule, builtins::BUILTINS, get_stdlib},
};

pub const MAX_DEPTH: usize = 2000;

pub struct VM {
    file: String,
    pub stack: Vec<Value>,
    frames: Vec<CallFrame>,
    open_upvalues: Vec<ArcUpvalue>, // track upvalues that point to stack slots
    global_table: HashMap<String, Value>,
    loaded_modules: HashMap<String, ArcModule>,
    preloaded_modules: HashMap<String, Arc<dyn LazyModule>>,
}

impl VM {
    pub fn new(file: String) -> Self {
        let builtins = BUILTINS.clone();
        let mut s = Self {
            file,
            stack: Vec::new(),
            frames: Vec::new(),
            open_upvalues: Vec::new(),
            global_table: builtins,
            loaded_modules: HashMap::new(),
            preloaded_modules: get_stdlib(),
        };

        // let texte_module = s.load_module("Texte").expect("The Texte module");
        // s.global_table.insert(String::from("Texte"), texte_module);

        s
    }

    pub fn insert_module(&mut self, name: impl ToString, module: Arc<dyn LazyModule>) {
        self.preloaded_modules.insert(name.to_string(), module);
    }

    // pub fn insert_module(&mut self, (name, module): (impl ToString, Arc<dyn LazyModule>)) {
    //     self.preloaded_modules.insert(name.to_string(), module);
    // }

    pub fn dump_stack(&self) -> String {
        format!("{:#?}", self.stack)
    }

    pub fn insert_global(&mut self, (name, val): (impl ToString, Value)) -> Option<Value> {
        self.global_table.insert(name.to_string(), val)
    }

    pub fn get_global(&mut self, name: &str) -> Option<&Value> {
        self.global_table.get(name)
    }

    fn push(&mut self, v: Value) {
        self.stack.push(v);
    }

    fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    fn peek(&self, distance: usize) -> &Value {
        let idx = self.stack.len() - 1 - distance;
        &self.stack[idx]
    }

    fn get_frame(&mut self) -> Result<&mut CallFrame, RuntimeError> {
        self.frames
            .last_mut()
            .ok_or(RuntimeError::generic_err("no frame"))
    }

    pub(crate) fn get_std_module(&mut self, name: &'static str) -> ArcModule {
        let _ = self.load_module(name);

        Arc::clone(&self.loaded_modules[name])
    }

    fn close_upvalues(&mut self, frame_base: usize) {
        // close any open upvalues that refer to stack slots >= frame_base
        // simple linear scan; remove closed ones
        let mut i = 0;
        while i < self.open_upvalues.len() {
            let refer = {
                let uv = self.open_upvalues[i].read().unwrap();
                match uv.location {
                    UpvalueLocation::Open(idx) => idx >= frame_base,
                    UpvalueLocation::Closed(_) => false,
                }
            };
            if refer {
                // close it
                self.open_upvalues[i].write().unwrap().close(self);
                // remove from open_upvalues
                self.open_upvalues.remove(i);
            } else {
                i += 1;
            }
        }
    }

    fn lookup_module(&mut self, module_name: &str) -> Result<ArcModule, RuntimeError> {
        let module_name = module_name.strip_suffix(".as").unwrap_or(module_name);

        if let Some(module) = self.loaded_modules.get(module_name) {
            return Ok(Arc::clone(&module));
        }

        if let Some(lazy_module) = self.preloaded_modules.get(module_name) {
            let module = lazy_module.load();

            self.loaded_modules
                .insert(module_name.to_string(), Arc::clone(&module));

            return Ok(module);
        }

        let module_file = Path::new(&self.file)
            .parent()
            .map(|p| format!("{}/{}.as", p.display(), module_name))
            .unwrap_or(format!("./{}.as", module_name));

        let abs_module_file = if let Ok(path) = fs::canonicalize(&module_file) {
            path.display().to_string()
        } else {
            let std_modules = env::var("ALIVESCRIPT_MODULES_STD").unwrap_or_else(|_| {
                let exe_folder = env::current_exe()
                    .map(|e| format!("{}/stdlib", e.parent().unwrap().display()))
                    .unwrap_or_default();
                format!("{}", vec![exe_folder].join(":"))
            });

            let search_dirs = env::var("ALIVESCRIPT_MODULES").or_else(|_| {
                let modules_folder = env::current_dir()
                    .map(|e| format!("{}/modules", e.display()))
                    .unwrap_or_default();
                Ok(format!("{}", vec![modules_folder].join(":")))
            })?;

            let modules_path = vec![std_modules, search_dirs].join(":");

            let mut found = None;
            for search_dir in modules_path.split(":") {
                let path = format!("{}/{}", search_dir, module_file);
                if let Ok(path) = fs::canonicalize(&path) {
                    found = Some(path.display().to_string());
                    break;
                }
            }

            if let Some(path) = found {
                path
            } else {
                return Err(RuntimeError::module_load_error(
                    module_name,
                    format!(
                        "fichier non trouvé dans le répertoire courant ou dans {}",
                        modules_path
                    ),
                ));
            }
        };

        let module_file_content = fs::read_to_string(&abs_module_file)
            .map_err(|io_err| RuntimeError::module_load_error(module_name, io_err))?;

        let compiler = Compiler::new(&module_file_content, module_file.to_string());
        let module_closure = compiler
            .parse_and_compile_to_module()
            .map_err(|err| RuntimeError::module_load_error(module_name, err))?;

        let mut other_vm = VM::new(module_file.to_string());
        other_vm.run(module_closure.load_fn)?;

        let mut members = HashMap::new();
        for (member_name, member) in module_closure.exported_members {
            let value = other_vm.stack[member.value_idx].clone();
            members.insert(
                member_name,
                ASField::new(member.is_const, member.field_type, value),
            );
        }

        let module = Arc::new(RwLock::new(ASModule {
            name: module_name.to_string(),
            members,
        }));

        self.loaded_modules
            .insert(module_name.to_string(), Arc::clone(&module));

        Ok(module)
    }

    fn load_module(&mut self, module_name: &str) -> Result<Value, RuntimeError> {
        let module = self.lookup_module(module_name)?;
        Ok(Value::Module(module))
    }

    fn finish_init_struct(
        &mut self,
        structure: ArcStructure,
        mut new_s_fields: HashMap<String, Value>,
    ) -> Result<HashMap<String, ASField>, RuntimeError> {
        let struct_name = structure.read().unwrap().name.clone();
        let mut final_fields = HashMap::with_capacity(structure.read().unwrap().fields.len());

        let mut missing = vec![];
        for field_info in structure.read().unwrap().fields.iter() {
            if let Some(value) = new_s_fields.remove(&field_info.name) {
                final_fields.insert(
                    field_info.name.clone(),
                    ASField {
                        value,
                        is_const: field_info.is_const,
                        field_type: field_info.field_type.clone().as_base_type().unwrap(),
                    },
                );
                continue;
            }

            if let Some(ref default_val) = field_info.value {
                // we don't eval if we will throw an error after anyway
                if !missing.is_empty() {
                    continue;
                }

                let Value::Function(Function::ClosureInst(factory)) = default_val else {
                    panic!(
                        "Dans la structure '{}'. La valeur par défaut du champs '{}' doit être une closure",
                        struct_name, field_info.name
                    )
                };

                let base = self.stack.len();
                self.frames.push(CallFrame {
                    closure: Arc::clone(factory),
                    ip: 0,
                    base,
                });
                self.check_overflow()?;
                let value = self.run_frame(self.frames.len() - 1)?;

                final_fields.insert(
                    field_info.name.clone(),
                    ASField {
                        value,
                        is_const: field_info.is_const,
                        field_type: field_info.field_type.clone().as_base_type().unwrap(),
                    },
                );
            } else {
                missing.push(field_info.name.clone());
            }
        }

        if !missing.is_empty() {
            return Err(RuntimeError::missing_struct_fields(&struct_name, &missing));
        }

        Ok(final_fields)
    }

    pub(crate) fn resolve_proto_closure_upvalues(
        &mut self,
        proto_closure: ArcClosureProto,
    ) -> Result<ArcClosureInst, RuntimeError> {
        let frame = self.get_frame()?;
        let function = proto_closure.function.clone();
        // build real closure and wire upvalues according to function.upvalue_specs
        let mut closure_upvalues: Vec<ArcUpvalue> = Vec::with_capacity(function.upvalue_count);

        let frame_base = frame.base;

        for spec in function.upvalue_specs.iter() {
            match spec {
                UpvalueSpec::Local(slot_idx) => {
                    // locate existing open upvalue for this exact stack slot (base + slot_idx)
                    let target_stack_idx = frame_base + slot_idx;

                    let existing = self.open_upvalues.iter().find(|uv| {
                        if let UpvalueLocation::Open(idx) = uv.read().unwrap().location {
                            idx == target_stack_idx
                        } else {
                            false
                        }
                    });

                    let rc_up = match existing {
                        Some(e) => e.clone(),
                        None => {
                            let uv = Upvalue {
                                location: UpvalueLocation::Open(target_stack_idx),
                            };
                            let rc = Arc::new(RwLock::new(uv));
                            self.open_upvalues.push(rc.clone());
                            rc
                        }
                    };
                    closure_upvalues.push(rc_up);
                }
                UpvalueSpec::Upvalue(parent_index) => {
                    // inherit parent's upvalue: parent's closure is frame.closure
                    let parent_uv = self
                        .get_frame()
                        .unwrap()
                        .closure
                        .upvalues
                        .get(*parent_index)
                        .ok_or(RuntimeError::generic_err(
                            "parent upvalue index out of range",
                        ))?;
                    closure_upvalues.push(parent_uv.clone());
                }
            }
        }
        let new_closure = Arc::new(ClosureInst::new(function.clone(), closure_upvalues));

        Ok(new_closure)
    }

    fn call_fn(&mut self, nbargs: usize, func: Value) -> Result<(), RuntimeError> {
        match func {
            Value::Function(Function::NativeFunction(f)) => {
                let mut args = VecDeque::with_capacity(nbargs);
                let f = f.clone();

                for _ in 0..nbargs {
                    args.push_front(self.pop().unwrap());
                }

                // remove the function that is still on the stack
                self.pop();

                let result = (f.func)(self, args.into())?;

                self.push(result.unwrap_or(Value::Nul));
            }
            Value::Function(Function::NativeMethod(f)) => {
                let mut args = VecDeque::with_capacity(nbargs);
                let f = f.clone();

                for _ in 0..nbargs {
                    args.push_front(self.pop().unwrap());
                }

                args.push_front(*f.inst_value);

                // remove the function that is still on the stack
                self.pop();

                let result = (f.func.func)(self, args.into())?;

                self.push(result.unwrap_or(Value::Nul));
            }
            Value::Function(Function::ClosureInst(closure)) => {
                if nbargs != closure.function.nb_params {
                    return Err(RuntimeError::call_error(
                        closure
                            .function
                            .name
                            .as_ref()
                            .unwrap_or(&String::from("anonyme")),
                        format!(
                            "mauvais nombre d'arguments (attendu {}, obtenu {})",
                            closure.function.nb_params, nbargs
                        ),
                    ));
                }
                // set the base as the first arg of the function
                let base = self.stack.len() - nbargs;
                self.frames.push(CallFrame {
                    closure: Arc::clone(&closure),
                    ip: 0,
                    base,
                });
                self.check_overflow()?;
            }
            Value::Function(Function::ClosureProto(closure)) => {
                if nbargs != closure.function.nb_params {
                    return Err(RuntimeError::call_error(
                        closure
                            .function
                            .name
                            .as_ref()
                            .unwrap_or(&String::from("anonyme")),
                        format!(
                            "mauvais nombre d'arguments (attendu {}, obtenu {})",
                            closure.function.nb_params, nbargs
                        ),
                    ));
                }
                let closure = self.resolve_proto_closure_upvalues(closure)?;
                // set the base as the first arg of the function
                let base = self.stack.len() - nbargs;
                self.frames.push(CallFrame {
                    closure,
                    ip: 0,
                    base,
                });
                self.check_overflow()?;
            }
            Value::Function(Function::ClosureMethod(method)) => {
                if nbargs + 1 != method.read().unwrap().closure.function.nb_params {
                    return Err(RuntimeError::call_error(
                        method
                            .read()
                            .unwrap()
                            .closure
                            .function
                            .name
                            .as_ref()
                            .unwrap_or(&String::from("anonyme")),
                        format!(
                            "mauvais nombre d'arguments (attendu {}, obtenu {})",
                            method.read().unwrap().closure.function.nb_params,
                            nbargs
                        ),
                    ));
                }
                // set the base as the first arg of the function
                let base = self.stack.len() - nbargs;
                let inst_value = method.read().unwrap().inst_value.clone();
                let closure = Arc::clone(&method.read().unwrap().closure);

                self.frames.push(CallFrame {
                    closure,
                    ip: 0,
                    base,
                });
                self.check_overflow()?;

                // setting the "inst" argument as the first argument
                self.stack.insert(base, inst_value);
            }
            _ => {
                return Err(RuntimeError::generic_err(format!(
                    "Cannot call value of type: {}",
                    func.get_type()
                )));
            }
        };
        Ok(())
    }

    /// This is an API to call an AliveScript function from code *outside* of AliveScript
    pub fn run_fn(&mut self, args: Vec<Value>, func: &Function) -> Result<Value, RuntimeError> {
        let nbargs = args.len();

        match func {
            Function::NativeFunction(f) => {
                let mut args = VecDeque::from_iter(args);
                let f = f.clone();

                let result = (f.func)(self, args.into())?;

                return Ok(result.unwrap_or(Value::Nul));
            }
            Function::NativeMethod(f) => {
                let mut args = VecDeque::from_iter(args);
                let f = f.clone();

                args.push_front(*f.inst_value);

                let result = (f.func.func)(self, args.into())?;

                return Ok(result.unwrap_or(Value::Nul));
            }
            Function::ClosureInst(closure) => {
                let curr_depth = self.frames.len();
                for arg in args.into_iter().rev() {
                    self.push(arg);
                }
                // set the base as the first arg of the function
                let base = self.stack.len() - nbargs;
                self.frames.push(CallFrame {
                    closure: Arc::clone(&closure),
                    ip: 0,
                    base,
                });
                self.check_overflow()?;

                return self.run_frame(curr_depth);
            }
            Function::ClosureProto(closure) => {
                let curr_depth = self.frames.len();
                for arg in args.into_iter().rev() {
                    self.push(arg)
                }

                let closure = self.resolve_proto_closure_upvalues(Arc::clone(closure))?;
                // set the base as the first arg of the function
                let base = self.stack.len() - nbargs;
                self.frames.push(CallFrame {
                    closure,
                    ip: 0,
                    base,
                });
                self.check_overflow()?;

                return self.run_frame(curr_depth);
            }
            Function::ClosureMethod(closure) => {
                let curr_depth = self.frames.len();
                for arg in args.into_iter().rev() {
                    self.push(arg)
                }

                // set the base as the first arg of the function
                let base = self.stack.len() - nbargs;
                let inst_value = closure.read().unwrap().inst_value.clone();
                let closure = Arc::clone(&closure.read().unwrap().closure);

                self.frames.push(CallFrame {
                    closure,
                    ip: 0,
                    base,
                });
                self.check_overflow()?;

                // setting the "inst" argument as the first argument
                self.stack.insert(base, inst_value);

                return self.run_frame(curr_depth);
            }
        };
    }

    pub fn check_overflow(&self) -> Result<(), RuntimeError> {
        if self.frames.len() >= MAX_DEPTH {
            Err(RuntimeError::stackoverflow_error(format!(
                "la pile d'appel a dépassé la limite autorisée ({})",
                MAX_DEPTH
            )))
        } else {
            Ok(())
        }
    }

    pub fn run(&mut self, closure: ClosureProto) -> Result<Value, RuntimeError> {
        self.run_main(Arc::new(ClosureInst::new(closure.function, vec![])))
    }

    pub fn run_file_to_module(&mut self, file_path: &str) -> Result<ArcModule, RuntimeError> {
        self.lookup_module(file_path)
    }

    fn run_main(&mut self, closure: ArcClosureInst) -> Result<Value, RuntimeError> {
        self.frames.push(CallFrame {
            closure,
            ip: 0,
            base: 0,
        });
        self.run_frame(self.frames.len() - 1)
    }

    fn run_frame(&mut self, until_depth: usize) -> Result<Value, RuntimeError> {
        loop {
            let frame = self.get_frame()?;
            let fnc = &frame.closure.function;
            if frame.ip >= fnc.code.len() {
                return Err(RuntimeError::generic_err("IP out of range"));
            }
            let op = Opcode::try_from(fnc.code[frame.ip]).expect("Value expected to be an opcode");
            frame.ip += 1;

            match op {
                Opcode::Pop => {
                    self.pop();
                }
                Opcode::Dup => {
                    self.push(self.peek(0).clone());
                }

                Opcode::Constant => {
                    let const_idx = fnc.code[frame.ip];
                    frame.ip += 1;
                    let val = fnc.constants[const_idx as usize].clone();
                    self.push(val);
                }
                Opcode::Neg => {
                    let val = self.pop().unwrap();
                    self.push((Value::Entier(0) - val)?);
                }
                Opcode::NewList => {
                    let nb_el = fnc.code[frame.ip];
                    frame.ip += 1;

                    let mut lst = VecDeque::with_capacity(nb_el as usize);
                    for _ in 0..nb_el {
                        lst.push_front(self.pop().unwrap());
                    }
                    self.push(Value::Liste(Arc::new(RwLock::new(lst.into()))).into());
                }
                Opcode::NewDict => {
                    let nb_el = fnc.code[frame.ip];
                    frame.ip += 1;

                    let mut members = HashMap::with_capacity(nb_el as usize);
                    for _ in 0..nb_el {
                        let field_name = self.pop().unwrap();
                        let field_value = self.pop().unwrap();
                        members.insert(field_name.to_string(), field_value);
                    }

                    self.push(Value::Dict(Arc::new(RwLock::new(ASDict { members }))).into());
                }
                Opcode::GetItem => {
                    let slice = self.pop().unwrap();
                    let obj = self.pop().unwrap();

                    let result = match (obj, slice) {
                        (Value::Dict(d), Value::Texte(txt)) => {
                            let d = d.read().unwrap();
                            let Some(member) = d.members.get(&txt) else {
                                return Err(RuntimeError::generic_err(format!(
                                    "Erreur de clé. La clé {} n'existe pas dans le dictionnaire.",
                                    Value::Texte(txt).repr()
                                )));
                            };

                            member.clone()
                        }
                        (Value::Dict(d), Value::Entier(i)) => {
                            let d = d.read().unwrap();
                            let Some(member) = d.members.get(&i.to_string()) else {
                                return Err(RuntimeError::generic_err(format!(
                                    "Erreur de clé. La clé {} n'existe pas dans le dictionnaire.",
                                    Value::Entier(i).repr()
                                )));
                            };

                            member.clone()
                        }
                        (Value::Dict(d), Value::Decimal(f)) => {
                            let d = d.read().unwrap();
                            let Some(member) = d.members.get(&f.to_string()) else {
                                return Err(RuntimeError::generic_err(format!(
                                    "Erreur de clé. La clé {} n'existe pas dans le dictionnaire.",
                                    Value::Decimal(f).repr()
                                )));
                            };

                            member.clone()
                        }
                        (Value::Liste(lst), Value::Entier(i)) => {
                            let lst = lst.read().unwrap();
                            let i = if i < 0 { lst.len() as i64 + i } else { i };
                            if i < 0 || i >= lst.len() as i64 {
                                // throw_err!(self, ASErreurType::new_erreur_index(i, lst.len()));
                            }
                            lst[i as usize].clone()
                        }
                        (Value::Liste(lst), Value::Decimal(i)) if i.fract() == 0.0 => {
                            let i = i as i64;
                            let lst = lst.read().unwrap();
                            let i = if i < 0 { lst.len() as i64 + i } else { i };
                            if i < 0 || i >= lst.len() as i64 {
                                // throw_err!(self, ASErreurType::new_erreur_index(i, lst.len()));
                            }
                            lst[i as usize].clone()
                        }
                        (Value::Liste(lst), Value::Liste(range)) => {
                            let range = range.read().unwrap();
                            let mut lst_final = Vec::with_capacity(range.len());
                            for obj in range.iter() {
                                if let Value::Entier(i) = obj {
                                    let lst = lst.read().unwrap();
                                    let i = if *i < 0 { lst.len() as i64 + *i } else { *i };
                                    if i < 0 || i >= lst.len() as i64 {
                                        // throw_err!(
                                        //     self,
                                        //     ASErreurType::new_erreur_index(i, lst.len())
                                        // );
                                    }
                                    lst_final.push(lst[i as usize].clone());
                                } else {
                                    // throw_err!(
                                    //     self,
                                    //     ASErreurType::new_erreur_type(
                                    //         ASType::Entier,
                                    //         obj.get_type()
                                    //     )
                                    // );
                                }
                            }
                            Value::Liste(Arc::new(RwLock::new(lst_final)))
                        }
                        (Value::Texte(txt), Value::Entier(i)) => {
                            let i = if i < 0 { txt.len() as i64 + i } else { i };
                            if i < 0 || i >= txt.len() as i64 {
                                // throw_err!(self, ASErreurType::new_erreur_index(i, txt.len()));
                            }
                            Value::Texte(txt[i as usize..i as usize + 1].into())
                        }
                        (Value::Texte(txt), Value::Liste(range)) => {
                            let range = range.read().unwrap();
                            let mut txt_final = String::with_capacity(range.len());
                            for obj in range.iter() {
                                if let Value::Entier(i) = obj {
                                    let i = if *i < 0 { txt.len() as i64 + *i } else { *i };
                                    if i < 0 || i >= txt.len() as i64 {
                                        // throw_err!(
                                        //     self,
                                        //     ASErreurType::new_erreur_index(i, txt.len())
                                        // );
                                    }
                                    txt_final.push(txt.chars().nth(i as usize).unwrap());
                                } else {
                                    // throw_err!(
                                    //     self,
                                    //     ASErreurType::new_erreur_type(
                                    //         ASType::Entier,
                                    //         obj.get_type(),
                                    //     )
                                    // );
                                }
                            }
                            Value::Texte(txt_final)
                        }
                        args => todo!(
                            "GET_ITEM on '{}' ({}) and '{}' ({})",
                            args.0,
                            args.0.get_type(),
                            args.1,
                            args.1.get_type()
                        ),
                    };

                    self.push(result);
                }
                Opcode::SetItem => {
                    let slice = self.pop().unwrap();
                    let obj = self.pop().unwrap();
                    let value = self.pop().unwrap();

                    match (obj, slice) {
                        (Value::Dict(d), Value::Texte(t)) => {
                            let mut dict = d.write().unwrap();
                            dict.members.insert(t, value);
                        }
                        (Value::Dict(d), Value::Entier(i)) => {
                            let mut dict = d.write().unwrap();
                            dict.members.insert(i.to_string(), value);
                        }
                        (Value::Dict(d), Value::Decimal(f)) => {
                            let mut dict = d.write().unwrap();
                            dict.members.insert(f.to_string(), value);
                        }
                        (Value::Liste(lst), Value::Entier(i)) => {
                            let mut lst = lst.write().unwrap();
                            let i = if i < 0 { lst.len() as i64 + i } else { i };
                            if i < 0 || i >= lst.len() as i64 {
                                return Err(RuntimeError::generic_err(format!(
                                    "Erreur d'accès. Taille de la liste: {}, mais l'index donnée est {}",
                                    lst.len(),
                                    i
                                )));
                            }
                            lst[i as usize] = value;
                        }
                        (Value::Liste(lst), Value::Decimal(i)) if i.fract() == 0.0 => {
                            let i = i as i64;
                            let mut lst = lst.write().unwrap();
                            let i = if i < 0 { lst.len() as i64 + i } else { i };
                            if i < 0 || i >= lst.len() as i64 {
                                return Err(RuntimeError::generic_err(format!(
                                    "Erreur d'accès. Taille de la liste: {}, mais l'index donnée est {}",
                                    lst.len(),
                                    i
                                )));
                            }
                            lst[i as usize] = value;
                        }
                        (Value::Liste(lst), Value::Liste(range)) => {
                            let range = range.read().unwrap();
                            let mut lst_final = Vec::with_capacity(range.len());
                            for obj in range.iter() {
                                if let Value::Entier(i) = obj {
                                    let lst = lst.read().unwrap();
                                    let i = if *i < 0 { lst.len() as i64 + *i } else { *i };
                                    if i < 0 || i >= lst.len() as i64 {
                                        // throw_err!(
                                        //     self,
                                        //     ASErreurType::new_erreur_index(i, lst.len())
                                        // );
                                    }
                                    lst_final.push(lst[i as usize].clone());
                                } else {
                                    // throw_err!(
                                    //     self,
                                    //     ASErreurType::new_erreur_type(
                                    //         ASType::Entier,
                                    //         obj.get_type()
                                    //     )
                                    // );
                                }
                            }
                        }
                        (Value::Texte(txt), Value::Entier(i)) => {
                            return Err(RuntimeError::generic_err(
                                "Impossible de modifier une chaîne de charactères, car elles sont immuables",
                            ));
                        }
                        args => todo!(
                            "SET_ITEM on '{}' ({}) and '{}' ({})",
                            args.0,
                            args.0.get_type(),
                            args.1,
                            args.1.get_type()
                        ),
                    };
                }
                Opcode::GetField => {
                    let field_const_idx = fnc.code[frame.ip] as usize;
                    frame.ip += 1;

                    let field_name = match &fnc.constants[field_const_idx] {
                        Value::Texte(t) => t.clone(),
                        f => {
                            return Err(RuntimeError::generic_err(format!(
                                "field name must be a string, got {}",
                                f
                            )));
                        }
                    };

                    match self.pop().expect("Object") {
                        Value::Objet(o) => {
                            let val = ASObjet::get_field(self, o, &field_name)?;
                            self.push(val);
                        }
                        Value::NativeObjet(o) => {
                            let val = o.get_member(self, &field_name)?;
                            self.push(val);
                        }
                        Value::Structure(s) => {
                            let structure = s.read().unwrap();
                            if let Some(method) = structure.methods.get(&field_name) {
                                self.push(Value::Function(Function::ClosureInst(Arc::clone(
                                    method,
                                ))));
                            } else {
                                return Err(RuntimeError::invalid_field(
                                    &structure.to_string(),
                                    &field_name,
                                ));
                            }
                        }
                        Value::Module(m) => {
                            let module = m.read().unwrap();
                            if let Some(field) = module.members.get(&field_name) {
                                self.push(field.value.clone());
                            } else {
                                return Err(RuntimeError::invalid_field(&module.name, &field_name));
                            }
                        }
                        Value::Texte(s) => {
                            let module = self.get_std_module("Texte");
                            let module = module.read().unwrap();
                            if let Some(ASField {
                                value: Value::Function(Function::NativeFunction(f)),
                                ..
                            }) = module.members.get(&field_name)
                            {
                                self.push(Value::Function(Function::NativeMethod(NativeMethod {
                                    func: f.clone(),
                                    inst_value: Box::new(Value::Texte(s)),
                                })))
                            } else {
                                return Err(RuntimeError::invalid_field(&module.name, &field_name));
                            }
                        }
                        Value::Liste(lst) => {
                            let module = self.get_std_module("Liste");
                            let module = module.read().unwrap();
                            if let Some(ASField {
                                value: Value::Function(Function::NativeFunction(f)),
                                ..
                            }) = module.members.get(&field_name)
                            {
                                self.push(Value::Function(Function::NativeMethod(NativeMethod {
                                    func: f.clone(),
                                    inst_value: Box::new(Value::Liste(lst)),
                                })))
                            } else {
                                return Err(RuntimeError::invalid_field(&module.name, &field_name));
                            }
                        }
                        Value::Dict(d) => {
                            let dict = d.read().unwrap();
                            if let Some(member) = dict.members.get(&field_name) {
                                self.push(member.clone());
                            } else {
                                drop(dict);
                                let module = self.get_std_module("Dict");
                                let module = module.read().unwrap();
                                if let Some(ASField {
                                    value: Value::Function(Function::NativeFunction(f)),
                                    ..
                                }) = module.members.get(&field_name)
                                {
                                    self.push(Value::Function(Function::NativeMethod(
                                        NativeMethod {
                                            func: f.clone(),
                                            inst_value: Box::new(Value::Dict(d)),
                                        },
                                    )))
                                } else {
                                    return Err(RuntimeError::invalid_field(
                                        &module.name,
                                        &field_name,
                                    ));
                                }
                            }
                        }
                        o => {
                            return Err(RuntimeError::type_error(format!(
                                "impossible d'accéder à un champs sur une valeur de type '{}'",
                                o.get_type()
                            )));
                        }
                    }
                }
                Opcode::SetField => {
                    let field_const_idx = fnc.code[frame.ip] as usize;
                    frame.ip += 1;

                    let field_name = match &fnc.constants[field_const_idx] {
                        Value::Texte(t) => t.clone(),
                        f => {
                            return Err(RuntimeError::generic_err(format!(
                                "field name must be a string, got {}",
                                f
                            )));
                        }
                    };

                    let obj = self.pop().expect("Object");
                    let value = self.pop().expect("Value");

                    match obj {
                        Value::Objet(o) => {
                            ASObjet::set_field(self, o, &field_name, value)?;
                        }
                        Value::NativeObjet(o) => {
                            o.set_member(self, &field_name, value)?;
                        }
                        Value::Structure(s) => {
                            return Err(RuntimeError::type_error(format!(
                                "impossible de changer le champs d'une structure puisqu'elles sont immuables",
                            )));
                        }
                        Value::Module(m) => m.write().unwrap().set_member(&field_name, value)?,
                        Value::Dict(d) => {
                            let mut dict = d.write().unwrap();
                            dict.members.insert(field_name, value);
                        }
                        o => {
                            return Err(RuntimeError::type_error(format!(
                                "impossible de changer le champs d'une valeur de type '{}'",
                                o.get_type()
                            )));
                        }
                    }
                }
                Opcode::SetMethod => {
                    let name_const_idx = fnc.code[frame.ip] as usize;
                    frame.ip += 1;

                    let name = fnc.constants[name_const_idx as usize]
                        .clone()
                        .as_texte()
                        .expect("The name must be a string")
                        .to_string();

                    let closure = match self.pop().unwrap() {
                        Value::Function(Function::ClosureInst(rc_cl)) => rc_cl.clone(),
                        _ => {
                            return Err(RuntimeError::generic_err(
                                "CLOSURE constant must be a Function wrapped as Closure",
                            ));
                        }
                    };

                    // we don't pop the structure
                    let Value::Structure(structure) = self.peek(0) else {
                        return Err(RuntimeError::type_error(format!(
                            "Une méthode peut seulement être définie dans une structure, pas dans une valeur de type '{}'",
                            self.peek(0).get_type()
                        )));
                    };

                    structure.write().unwrap().methods.insert(name, closure);
                }
                Opcode::SetDefaultField => {
                    let name_const_idx = fnc.code[frame.ip] as usize;
                    frame.ip += 1;

                    let name = fnc.constants[name_const_idx as usize]
                        .clone()
                        .as_texte()
                        .expect("The name must be a string")
                        .to_string();

                    let closure = match self.pop().unwrap() {
                        Value::Function(Function::ClosureInst(rc_cl)) => rc_cl.clone(),
                        _ => {
                            return Err(RuntimeError::generic_err(
                                "CLOSURE constant must be a Function wrapped as Closure",
                            ));
                        }
                    };

                    // we don't pop the structure
                    let Value::Structure(structure) = self.peek(0) else {
                        return Err(RuntimeError::type_error(format!(
                            "Une méthode peut seulement être définie dans une structure, pas dans une valeur de type '{}'",
                            self.peek(0).get_type()
                        )));
                    };

                    for field in structure.write().unwrap().fields.iter_mut() {
                        if field.name == name {
                            field.value = Some(Value::Function(Function::ClosureInst(closure)));
                            break;
                        }
                    }
                }

                Opcode::Closure => {
                    let const_idx = fnc.code[frame.ip] as usize;
                    frame.ip += 1;
                    // We assume the constant is actually a Function wrapped as Value::Closure sentinel:
                    // convention: constants[const_idx] stores a Value::Closure whose inner `function` is an RcFunction with upvalue_specs populated.
                    // To keep constants simple, we actually store the function as a Closure inside the constant with no upvalues.
                    let proto_closure = match &fnc.constants[const_idx] {
                        Value::Function(Function::ClosureProto(rc_cl)) => rc_cl.clone(),
                        _ => {
                            return Err(RuntimeError::generic_err(
                                "CLOSURE constant must be a Function wrapped as Closure",
                            ));
                        }
                    };

                    let new_closure = self.resolve_proto_closure_upvalues(proto_closure)?;
                    self.push(Value::Function(Function::ClosureInst(new_closure)));
                }

                Opcode::NewStruct => {
                    let nb_fields = fnc.code[frame.ip];
                    frame.ip += 1;

                    let mut fields = HashMap::with_capacity(nb_fields as usize);
                    for _ in 0..nb_fields {
                        let field_name = self
                            .pop()
                            .unwrap()
                            .as_texte()
                            .expect("Field name must be a string")
                            .to_string();
                        let field_value = self.pop().unwrap();
                        fields.insert(field_name, field_value);
                    }

                    let maybe_struct = self.pop().expect("A struct");
                    let Value::Structure(struct_const) = maybe_struct else {
                        return Err(RuntimeError::invalid_struct(maybe_struct.get_type()));
                    };

                    let fields = self.finish_init_struct(Arc::clone(&struct_const), fields)?;

                    self.push(Value::Objet(
                        RwLock::new(ASObjet::new(struct_const, fields)).into(),
                    ));
                }

                Opcode::GetUpvalue => {
                    let idx = fnc.code[frame.ip] as usize;
                    frame.ip += 1;
                    let uv = Arc::clone(
                        frame
                            .closure
                            .upvalues
                            .get(idx)
                            .ok_or(RuntimeError::generic_err("get upvalue out of range"))?,
                    );
                    let v = uv.read().unwrap().get(self);
                    self.push(v);
                }
                Opcode::SetUpvalue => {
                    let idx = fnc.code[frame.ip] as usize;
                    frame.ip += 1;
                    let up_rc = frame
                        .closure
                        .upvalues
                        .get(idx)
                        .ok_or(RuntimeError::generic_err("set upvalue out of range"))?
                        .clone();

                    let val = self.pop().unwrap_or(Value::Nul);
                    up_rc.write().unwrap().set(self, val);
                }

                Opcode::GetLocal => {
                    let slot = fnc.code[frame.ip] as usize;
                    frame.ip += 1;
                    let ip = frame.ip;
                    let idx = frame.base + slot;

                    let v = self
                        .stack
                        .get(idx)
                        .cloned()
                        .ok_or(RuntimeError::generic_err(format!(
                            "Variable sans valeur (slot={})\n{}\n",
                            idx,
                            CallFrame::get_trace(&self.frames),
                        )))?;

                    self.push(v);
                }
                Opcode::SetLocal => {
                    let slot = fnc.code[frame.ip] as usize;
                    frame.ip += 1;
                    let idx = frame.base + slot;

                    let val = self
                        .pop()
                        .ok_or(RuntimeError::generic_err("Missing value in SET_LOCAL"))?;
                    if idx >= self.stack.len() {
                        // expand stack to fit local (for simplicity)
                        self.stack.resize(idx + 1, Value::Nul);
                    }
                    self.stack[idx] = val;
                }

                Opcode::GetGlobal => {
                    let slot = fnc.code[frame.ip] as usize;
                    frame.ip += 1;

                    let Value::Texte(ref name) = fnc.constants[slot] else {
                        panic!("Name of global variable must be a string");
                    };
                    let name = name.clone();

                    let v = self.global_table.get(&name).cloned().ok_or_else(|| {
                        RuntimeError::generic_err(format!("Variable inconnue {:?}", name))
                    })?;

                    self.push(v);
                }
                Opcode::SetGlobal => {
                    let slot = fnc.code[frame.ip] as usize;
                    frame.ip += 1;

                    let Value::Texte(ref name) = fnc.constants[slot] else {
                        panic!("Name of global variable must be a string");
                    };
                    let name = name.clone();

                    let val = self
                        .pop()
                        .ok_or(RuntimeError::generic_err("Missing value in SET_LOCAL"))?;

                    self.global_table.insert(name, val);
                }

                Opcode::Call => {
                    let nbargs = fnc.code[frame.ip] as usize;
                    frame.ip += 1;

                    // get the function
                    let func = self.peek(nbargs).clone();

                    self.call_fn(nbargs, func)?;
                }
                Opcode::Return => {
                    let frame = self
                        .frames
                        .pop()
                        .ok_or(RuntimeError::generic_err("return with no frame"))?;

                    self.close_upvalues(frame.base);

                    // we don't truncate if self.frames is empty to allow for
                    // this vm to become a module
                    if self.frames.is_empty() {
                        return Ok(Value::Nul);
                    }

                    let ret = if self.stack.len() > frame.base {
                        self.pop().unwrap_or(Value::Nul)
                    } else {
                        Value::Nul
                    };

                    // remove stack entries above base (leave space where callee was)
                    self.stack.truncate(frame.base);

                    if self.frames.len() == until_depth {
                        return Ok(ret);
                    }

                    self.stack[frame.base - 1] = ret;
                }

                Opcode::BinOp => {
                    let op = fnc.code[frame.ip];
                    frame.ip += 1;

                    let binop = BinOpcode::try_from(op).expect(&format!("Invalid binop: {}", op));

                    let arg2 = self.pop().ok_or_else(|| {
                        RuntimeError::generic_err(format!("Missing rhs in {:?}", op))
                    })?;

                    let arg1 = self.pop().ok_or_else(|| {
                        RuntimeError::generic_err(format!("Missing rhs in {:?}", op))
                    })?;

                    self.push(match binop {
                        BinOpcode::Mul => arg1 * arg2,
                        BinOpcode::Div => arg1 / arg2,
                        BinOpcode::DivInt => arg1.div_int(arg2),
                        BinOpcode::Add => arg1 + arg2,
                        BinOpcode::Sub => arg1 - arg2,
                        BinOpcode::Exp => arg1.pow(arg2),
                        BinOpcode::Mod => arg1 % arg2,
                        BinOpcode::Extend => arg1.extend(arg2),
                        BinOpcode::BitwiseOr => arg1 | arg2,
                        BinOpcode::BitwiseAnd => arg1 & arg2,
                        BinOpcode::BitwiseXor => arg1 ^ arg2,
                        BinOpcode::ShiftLeft => arg1 << arg2,
                        BinOpcode::ShiftRight => arg1 >> arg2,
                    }?);
                }
                Opcode::BinComp => {
                    let op = fnc.code[frame.ip];
                    frame.ip += 1;

                    let binop = BinCompcode::try_from(op).expect(&format!("Invalid binop: {}", op));

                    let arg2 = self.pop().ok_or_else(|| {
                        RuntimeError::generic_err(format!("Missing rhs in {:?}", op))
                    })?;

                    let arg1 = self.pop().ok_or_else(|| {
                        RuntimeError::generic_err(format!("Missing rhs in {:?}", op))
                    })?;

                    self.push(
                        Value::Booleen(match binop {
                            BinCompcode::Eq => arg1 == arg2,
                            BinCompcode::NotEq => arg1 != arg2,
                            BinCompcode::Lth => arg1 < arg2,
                            BinCompcode::Gth => arg1 > arg2,
                            BinCompcode::Geq => arg1 >= arg2,
                            BinCompcode::Leq => arg1 <= arg2,
                            BinCompcode::Dans => arg2.contains(&arg1)?,
                            BinCompcode::PasDans => !arg2.contains(&arg1)?,
                        })
                        .into(),
                    );
                }
                Opcode::Not => {
                    let val = self.pop().expect("val");
                    self.push(Value::Booleen(!val.to_bool()));
                }

                Opcode::Jump => {
                    let dist = fnc.code[frame.ip];
                    let dist = dist as i16 - JUMP_OFFSET;
                    frame.ip = (frame.ip as i16 + dist + 1) as usize;
                }
                Opcode::JumpIfFalse => {
                    let dist = fnc.code[frame.ip];
                    let dist = dist as i16 - JUMP_OFFSET;
                    frame.ip += 1;

                    let ip = frame.ip;

                    let val = self.pop().expect("A value");

                    if !val.to_bool() {
                        self.get_frame().unwrap().ip = (ip as i16 + dist) as usize;
                    }
                }
                Opcode::JumpTest => {
                    let cond = fnc.code[frame.ip] == 1;
                    frame.ip += 1;
                    let dist = fnc.code[frame.ip];
                    let dist = dist as i16 - JUMP_OFFSET;
                    frame.ip += 1;

                    let ip = frame.ip;

                    let val = self.peek(0);

                    if val.to_bool() == cond {
                        self.get_frame().unwrap().ip = (ip as i16 + dist) as usize;
                    } else {
                        self.pop();
                    }
                }

                Opcode::LoadModule => {
                    let module_name_idx = fnc.code[frame.ip];
                    frame.ip += 1;

                    let Value::Texte(module_name) = fnc.constants[module_name_idx as usize].clone()
                    else {
                        return Err(RuntimeError::type_error(
                            "le nom d'un module doit être un texte",
                        ));
                    };

                    let module = self.load_module(&module_name)?;
                    self.push(module);
                }

                Opcode::Read | Opcode::ReadWithMsg => {
                    let msg = if op == Opcode::ReadWithMsg {
                        let msg = self.pop().expect("A message");
                        msg.to_string()
                    } else {
                        String::from("> ")
                    };

                    print!("{}", msg);
                    _ = io::stdout().flush();

                    let mut buf = String::new();
                    stdin()
                        .read_line(&mut buf)
                        .map_err(|err| RuntimeError::generic_err(err.to_string()))?;

                    // remove the \n
                    _ = buf.pop();

                    self.push(Value::Texte(buf));
                }
                Opcode::ReadCall | Opcode::ReadCallWithMsg => {
                    let lire_sinon_dist = fnc.code[frame.ip] as i16 - JUMP_OFFSET;
                    frame.ip += 1;

                    let msg = if op == Opcode::ReadCallWithMsg {
                        let msg = self.pop().expect("A message");
                        msg.to_string()
                    } else {
                        String::from("> ")
                    };

                    print!("{}", msg);
                    _ = io::stdout().flush();

                    let mut buf = String::new();
                    stdin()
                        .read_line(&mut buf)
                        .map_err(|err| RuntimeError::generic_err(err.to_string()))?;

                    // remove the \n
                    _ = buf.pop();

                    let func = self.peek(0).clone();

                    let len = self.stack.len();

                    self.push(Value::Texte(buf));
                    let result = self.call_fn(1, func);

                    if result.is_err() {
                        self.stack.truncate(len);
                        // if the function doesn't work -> go to "sinon"
                        let frame = self.get_frame().unwrap();
                        frame.ip = (frame.ip as i16 + lire_sinon_dist) as usize;
                    }
                }
                Opcode::TryCall => {
                    let nb_args = fnc.code[frame.ip] as usize;
                    frame.ip += 1;

                    let catch_dist = fnc.code[frame.ip] as i16 - JUMP_OFFSET;
                    frame.ip += 1;

                    let Value::Function(func) = self.peek(nb_args).clone() else {
                        unreachable!()
                    };

                    let mut args = vec![];
                    for _ in 0..nb_args {
                        args.push(self.pop().unwrap());
                    }

                    // the len w/o the args and the function
                    let len = self.stack.len() - 1;
                    let len_frames = self.frames.len();

                    let result = self.run_fn(args, &func);

                    match result {
                        Err(_) => {
                            self.stack.truncate(len);
                            self.frames.truncate(len_frames);
                            // if the function doesn't work -> go to "sinon"
                            let frame = self.get_frame().unwrap();
                            frame.ip = (frame.ip as i16 + catch_dist) as usize;
                        }
                        Ok(val) => {
                            // we discard the function
                            self.pop();
                            self.push(val);
                        }
                    }
                }

                Opcode::ForNext => {
                    let iter_var_idx = fnc.code[frame.ip];
                    frame.ip += 1;
                    let base = frame.base;

                    let iter_val = self.stack[base + iter_var_idx as usize].clone();
                    let iter_state_val = &self.stack[base + iter_var_idx as usize + 1];

                    let next_state = match iter_val {
                        Value::Texte(txt) => {
                            let state_idx = iter_state_val.as_entier().map_err(|_| {
                                RuntimeError::generic_err(format!(
                                    "il faut que l'état soit une entier, pas '{}'",
                                    iter_state_val.get_type()
                                ))
                            })?;

                            self.stack[base + iter_var_idx as usize + 1] =
                                Value::Entier(state_idx + 1);

                            if state_idx as usize >= txt.len() {
                                None
                            } else {
                                Some(Value::Texte(
                                    txt[state_idx as usize..state_idx as usize + 1].to_string(),
                                ))
                            }
                        }
                        Value::Liste(lst) => {
                            let state_idx = iter_state_val.as_entier().map_err(|_| {
                                RuntimeError::generic_err(format!(
                                    "il faut que l'état soit une entier, pas '{}'",
                                    iter_state_val.get_type()
                                ))
                            })?;

                            self.stack[base + iter_var_idx as usize + 1] =
                                Value::Entier(state_idx + 1);

                            if state_idx as usize >= lst.read().unwrap().len() {
                                None
                            } else {
                                Some(lst.read().unwrap()[state_idx as usize].clone())
                            }
                        }
                        Value::Function(f) => {
                            // let result = self.run_fn(1, &f);
                            todo!()
                        }
                        Value::Objet(rw_lock) => todo!(),
                        _ => {
                            return Err(RuntimeError::generic_err(format!(
                                "impossible d'itérer sur une valeur de type '{}'",
                                iter_val.get_type()
                            )));
                        }
                    };

                    if let Some(next_state) = next_state {
                        self.push(next_state);
                        // skip the jump
                        self.get_frame().unwrap().ip += 2;
                    }
                }
            }
        }
    }

    pub fn file(&self) -> &str {
        &self.file
    }
}
