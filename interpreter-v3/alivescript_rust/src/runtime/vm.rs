use std::{
    collections::{HashMap, VecDeque},
    fs,
    io::{self, Write, stdin},
    path::{self, Path},
    sync::{Arc, RwLock},
};

use crate::{
    compiler::{
        Compiler,
        bytecode::{BinCompcode, BinOpcode, JUMP_OFFSET, Opcode},
        obj::{ArcUpvalue, CallFrame, Function, Upvalue, UpvalueLocation, UpvalueSpec, Value},
        value::{
            ASModule, ASObjet, ArcClosureInst, ArcClosureProto, ArcModule, ArcStructure,
            ClosureInst, ClosureProto,
        },
    },
    runtime::{
        err::RuntimeError,
        module::{self, BUILTIN_MOD},
    },
};

pub struct VM {
    file: String,
    pub stack: Vec<Value>,
    frames: Vec<CallFrame>,
    open_upvalues: Vec<ArcUpvalue>, // track upvalues that point to stack slots
    global_table: HashMap<String, Value>,
    loaded_modules: HashMap<String, ArcModule>,
}

impl VM {
    pub fn new(file: String) -> Self {
        let builtin = BUILTIN_MOD.borrow().clone();

        Self {
            file,
            stack: Vec::new(),
            frames: Vec::new(),
            open_upvalues: Vec::new(),
            global_table: builtin,
            loaded_modules: HashMap::new(),
        }
    }

    pub fn insert_module(&mut self, name: impl ToString, module: ArcModule) {
        self.loaded_modules.insert(name.to_string(), module);
    }

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

    fn load_module(&mut self, module_name: &str) -> Result<Value, RuntimeError> {
        let module_name = module_name.strip_suffix(".as").unwrap_or(module_name);

        if let Some(module) = self.loaded_modules.get(module_name) {
            return Ok(Value::Module(Arc::clone(&module)));
        }

        let module_file = Path::new(&self.file)
            .parent()
            .unwrap()
            .join(format!("{}.as", module_name));

        let abs_module_file = fs::canonicalize(&module_file)
            .map_err(|io_err| RuntimeError::module_load_error(module_name, io_err))?;
        let module_file_content = fs::read_to_string(&abs_module_file)
            .map_err(|io_err| RuntimeError::module_load_error(module_name, io_err))?;

        let compiler = Compiler::new(&module_file_content, module_file.display().to_string());
        let module_closure = compiler
            .parse_and_compile_to_module()
            .map_err(|err| RuntimeError::module_load_error(module_name, err))?;

        let mut other_vm = VM::new(module_file.display().to_string());
        other_vm.run(module_closure.load_fn)?;

        let mut members = HashMap::new();
        for (member_name, member_idx) in module_closure.exported_members {
            let value = other_vm.stack[member_idx].clone();
            members.insert(member_name, value);
        }

        let module = Arc::new(RwLock::new(ASModule {
            name: module_name.to_string(),
            members,
        }));

        self.loaded_modules
            .insert(module_name.to_string(), Arc::clone(&module));

        Ok(Value::Module(module))
    }

    fn finish_init_struct(
        &mut self,
        structure: ArcStructure,
        mut new_s_fields: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, RuntimeError> {
        let struct_name = structure.read().unwrap().name.clone();
        let mut missing = vec![];
        for field_info in structure.read().unwrap().fields.iter() {
            if new_s_fields.contains_key(&field_info.name) {
                continue;
            }

            if let Some(ref default_val) = field_info.value {
                // we don't eval if we will throw an error after anyway
                if !missing.is_empty() {
                    continue;
                }

                let Value::Function(Function::ClosureProto(factory)) = default_val else {
                    panic!(
                        "Dans la structure '{}'. La valeur par défaut du champs '{}' doit être une closure",
                        struct_name, field_info.name
                    )
                };

                let new_closure = self.resolve_proto_closure_upvalues(Arc::clone(factory))?;
                let base = self.stack.len() - 1;
                self.frames.push(CallFrame {
                    closure: new_closure,
                    ip: 0,
                    base,
                });
                let value = self.run_frame(self.frames.len() - 1)?;

                new_s_fields.insert(field_info.name.clone(), value);
            } else {
                missing.push(field_info.name.clone());
            }
        }

        if !missing.is_empty() {
            return Err(RuntimeError::missing_struct_fields(&struct_name, &missing));
        }

        Ok(new_s_fields)
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

    pub fn call_fn(&mut self, nbargs: usize, func: Value) -> Result<(), RuntimeError> {
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
            Value::Function(Function::ClosureInst(closure)) => {
                // set the base as the first arg of the function
                let base = self.stack.len() - nbargs;
                self.frames.push(CallFrame {
                    closure: Arc::clone(&closure),
                    ip: 0,
                    base,
                });
            }
            Value::Function(Function::ClosureProto(closure)) => {
                let closure = self.resolve_proto_closure_upvalues(closure)?;
                // set the base as the first arg of the function
                let base = self.stack.len() - nbargs;
                self.frames.push(CallFrame {
                    closure,
                    ip: 0,
                    base,
                });
            }
            Value::Function(Function::ClosureMethod(closure)) => {
                // set the base as the first arg of the function
                let base = self.stack.len() - nbargs; // - 1 for the 'inst' param
                let inst_value = closure.read().unwrap().inst_value.clone();
                let closure = Arc::clone(&closure.read().unwrap().closure);

                self.frames.push(CallFrame {
                    closure,
                    ip: 0,
                    base,
                });

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

    pub fn run(&mut self, closure: ClosureProto) -> Result<Value, RuntimeError> {
        self.run_main(Arc::new(ClosureInst::new(closure.function, vec![])))
    }

    pub fn run_main(&mut self, closure: ArcClosureInst) -> Result<Value, RuntimeError> {
        self.frames.push(CallFrame {
            closure,
            ip: 0,
            base: 0,
        });
        self.run_frame(0)
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
                Opcode::GetItem => {
                    let slice = self.pop().unwrap();
                    let obj = self.pop().unwrap();

                    let result = match (obj, slice) {
                        (Value::Liste(lst), Value::Entier(i)) => {
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
                        _ => todo!(),
                    };

                    self.push(result);
                }
                Opcode::SetItem => {
                    todo!()
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
                        Value::Structure(s) => {
                            let structure = s.read().unwrap();
                            if let Some(method) = structure.struct_methods.get(&field_name) {
                                let new_method =
                                    self.resolve_proto_closure_upvalues(Arc::clone(method))?;
                                self.push(Value::Function(Function::ClosureInst(new_method)));
                            } else {
                                return Err(RuntimeError::invalid_field(
                                    &structure.to_string(),
                                    &field_name,
                                ));
                            }
                        }
                        Value::Module(m) => {
                            let module = m.read().unwrap();
                            if let Some(value) = module.members.get(&field_name) {
                                self.push(value.clone());
                            } else {
                                return Err(RuntimeError::invalid_field(&module.name, &field_name));
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
                    todo!()
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
                            "Variable sans valeur (ip={}, idx={})",
                            ip - 1,
                            idx
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
                        RuntimeError::generic_err(format!("Variable globale inconnue {:?}", name))
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

                Opcode::Pop => {
                    self.pop();
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
            }
        }
    }
}
