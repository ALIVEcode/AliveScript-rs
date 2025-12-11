use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    as_modules::ASModuleBuiltin,
    as_obj::{ASEnv, ASObj, ASType},
    ast::{BinCompcode, BinOpcode, CallRust},
    compiler::{
        bytecode::Opcode,
        module::BUILTIN_MOD,
        obj::{
            CallFrame, Closure, NativeFunction, RcClosure, RcUpvalue, Upvalue, UpvalueLocation,
            UpvalueSpec, Value,
        },
    },
};

pub struct VM {
    pub stack: Vec<Value>,
    frames: Vec<CallFrame>,
    open_upvalues: Vec<RcUpvalue>, // track upvalues that point to stack slots
    global_table: HashMap<String, Value>,
}

impl VM {
    pub fn new() -> Self {
        let builtin = BUILTIN_MOD.borrow().clone();

        Self {
            stack: Vec::new(),
            frames: Vec::new(),
            open_upvalues: Vec::new(),
            global_table: builtin,
        }
    }

    fn push(&mut self, v: Value) {
        self.stack.push(v);
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    fn peek(&self, distance: usize) -> &Value {
        let idx = self.stack.len() - 1 - distance;
        &self.stack[idx]
    }

    fn get_frame(&mut self) -> Result<&mut CallFrame, String> {
        self.frames.last_mut().ok_or("no frame".into())
    }

    fn close_upvalues(&mut self, frame_base: usize) {
        // close any open upvalues that refer to stack slots >= frame_base
        // simple linear scan; remove closed ones
        let mut i = 0;
        while i < self.open_upvalues.len() {
            let refer = {
                let uv = self.open_upvalues[i].borrow();
                match uv.location {
                    UpvalueLocation::Open(idx) => idx >= frame_base,
                    UpvalueLocation::Closed(_) => false,
                }
            };
            if refer {
                // close it
                self.open_upvalues[i].borrow_mut().close(self);
                // remove from open_upvalues
                self.open_upvalues.remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn run(&mut self, closure: RcClosure) -> Result<Value, String> {
        self.frames.push(CallFrame {
            closure: closure.clone(),
            ip: 0,
            base: 0,
        });
        loop {
            let frame = self.get_frame()?;
            let fnc = &frame.closure.function;
            if frame.ip >= fnc.code.len() {
                return Err("IP out of range".into());
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
                Opcode::Closure => {
                    let const_idx = fnc.code[frame.ip] as usize;
                    frame.ip += 1;
                    // We assume the constant is actually a Function wrapped as Value::Closure sentinel:
                    // convention: constants[const_idx] stores a Value::Closure whose inner `function` is an RcFunction with upvalue_specs populated.
                    // To keep constants simple, we actually store the function as a Closure inside the constant with no upvalues.
                    let proto_closure = match &fnc.constants[const_idx] {
                        Value::Closure(rc_cl) => rc_cl.clone(),
                        _ => {
                            return Err(
                                "CLOSURE constant must be a Function wrapped as Closure".into()
                            )
                        }
                    };
                    let function = proto_closure.function.clone();
                    // build real closure and wire upvalues according to function.upvalue_specs
                    let mut closure_upvalues: Vec<RcUpvalue> =
                        Vec::with_capacity(function.upvalue_count);

                    let frame_base = frame.base;

                    for spec in function.upvalue_specs.iter() {
                        match spec {
                            UpvalueSpec::Local(slot_idx) => {
                                // locate existing open upvalue for this exact stack slot (base + slot_idx)
                                let target_stack_idx = frame_base + slot_idx;

                                let existing = self.open_upvalues.iter().find(|uv| {
                                    if let UpvalueLocation::Open(idx) = uv.borrow().location {
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
                                        let rc = Rc::new(RefCell::new(uv));
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
                                    .ok_or("parent upvalue index out of range")?;
                                closure_upvalues.push(parent_uv.clone());
                            }
                        }
                    }
                    let new_closure = Rc::new(Closure {
                        function: function.clone(),
                        upvalues: closure_upvalues,
                    });
                    self.push(Value::Closure(new_closure));
                }
                Opcode::GetUpvalue => {
                    let idx = fnc.code[frame.ip] as usize;
                    frame.ip += 1;
                    let uv = Rc::clone(
                        frame
                            .closure
                            .upvalues
                            .get(idx)
                            .ok_or("get upvalue out of range")?,
                    );
                    let v = uv.borrow().get(self);
                    self.push(v);
                }
                Opcode::SetUpvalue => {
                    let idx = fnc.code[frame.ip] as usize;
                    frame.ip += 1;
                    let up_rc = frame
                        .closure
                        .upvalues
                        .get(idx)
                        .ok_or("set upvalue out of range")?
                        .clone();

                    let val = self.pop().unwrap_or(Value::ASObj(ASObj::ASNul));
                    up_rc.borrow_mut().set(self, val);
                }
                Opcode::GetLocal => {
                    let slot = fnc.code[frame.ip] as usize;
                    frame.ip += 1;
                    let idx = frame.base + slot;
                    let v = self
                        .stack
                        .get(idx)
                        .cloned()
                        .unwrap_or(ASObj::ASNoValue.into());
                    self.push(v);
                }
                Opcode::SetLocal => {
                    let slot = fnc.code[frame.ip] as usize;
                    frame.ip += 1;
                    let idx = frame.base + slot;

                    let val = self.pop().ok_or("Missing value in SET_LOCAL")?;
                    if idx >= self.stack.len() {
                        // expand stack to fit local (for simplicity)
                        self.stack.resize(idx + 1, ASObj::ASNoValue.into());
                    }
                    self.stack[idx] = val;
                }
                Opcode::GetGlobal => {
                    let slot = fnc.code[frame.ip] as usize;
                    frame.ip += 1;

                    let Value::ASObj(ASObj::ASTexte(ref name)) = fnc.constants[slot] else {
                        panic!("Name of global variable must be a string");
                    };
                    let name = name.clone();

                    let v = self
                        .global_table
                        .get(&name)
                        .cloned()
                        .ok_or_else(|| format!("Variable globale inconnue {:?}", name))?;

                    self.push(v);
                }
                Opcode::SetGlobal => {
                    let slot = fnc.code[frame.ip] as usize;
                    frame.ip += 1;

                    let Value::ASObj(ASObj::ASTexte(ref name)) = fnc.constants[slot] else {
                        panic!("Name of global variable must be a string");
                    };
                    let name = name.clone();

                    let val = self.pop().ok_or("Missing value in SET_LOCAL")?;

                    self.global_table.insert(name, val);
                }
                Opcode::Call => {
                    let nbargs = fnc.code[frame.ip] as usize;
                    frame.ip += 1;

                    // get the function
                    let func = self.peek(nbargs);

                    match func {
                        Value::NativeFunction(f) => {
                            let result = (f.clone().func)(self).map_err(|e| e.to_string())?;
                            for _ in 0..nbargs {
                                self.pop();
                            }
                            self.push(result.unwrap_or(Value::ASObj(ASObj::ASNoValue)));
                        }
                        Value::ASObj(asobj) => {
                            return Err(format!("Cannot call value of type: {}", asobj.get_type()));
                        }
                        Value::Closure(closure) => {
                            // set the base as the first arg of the function
                            let base = self.stack.len() - nbargs;
                            self.frames.push(CallFrame {
                                closure: Rc::clone(&closure),
                                ip: 0,
                                base,
                            });
                        }
                    }
                }
                Opcode::Return => {
                    let ret = self.pop().unwrap_or(ASObj::ASNul.into());

                    let frame = self.frames.pop().ok_or("return with no frame")?;
                    self.close_upvalues(frame.base);
                    // remove stack entries above base (leave space where callee was)
                    self.stack.truncate(frame.base);

                    if self.frames.is_empty() {
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

                    let Value::ASObj(arg2) = self
                        .pop()
                        .ok_or_else(|| format!("Missing rhs in {:?}", op))?
                    else {
                        return Err("Cannot do arithmetics on closures".into());
                    };
                    let Value::ASObj(arg1) = self
                        .pop()
                        .ok_or_else(|| format!("Missing lhs in {:?}", op))?
                    else {
                        return Err("Cannot do arithmetics on closures".into());
                    };

                    self.push(
                        match binop {
                            BinOpcode::Mul => arg1 * arg2,
                            BinOpcode::Div => arg1 / arg2,
                            BinOpcode::DivInt => arg1.div_int(arg2),
                            BinOpcode::Add => arg1 + arg2,
                            BinOpcode::Sub => arg1 - arg2,
                            BinOpcode::Exp => arg1.pow(arg2),
                            BinOpcode::Mod => arg1 % arg2,
                            BinOpcode::Extend => {
                                arg1.extend(arg2).map_err(|err| err.to_string())?
                            }
                            BinOpcode::BitwiseOr => (arg1 | arg2).map_err(|err| err.to_string())?,
                            BinOpcode::BitwiseAnd => {
                                (arg1 & arg2).map_err(|err| err.to_string())?
                            }
                            BinOpcode::BitwiseXor => {
                                (arg1 ^ arg2).map_err(|err| err.to_string())?
                            }
                            BinOpcode::ShiftLeft => {
                                (arg1 << arg2).map_err(|err| err.to_string())?
                            }
                            BinOpcode::ShiftRight => {
                                (arg1 >> arg2).map_err(|err| err.to_string())?
                            }
                        }
                        .into(),
                    );
                }
                Opcode::BinComp => {
                    let op = fnc.code[frame.ip];
                    frame.ip += 1;

                    let binop = BinCompcode::try_from(op).expect(&format!("Invalid binop: {}", op));

                    let Value::ASObj(arg2) = self
                        .pop()
                        .ok_or_else(|| format!("Missing rhs in {:?}", op))?
                    else {
                        return Err("Cannot do arithmetics on closures".into());
                    };
                    let Value::ASObj(arg1) = self
                        .pop()
                        .ok_or_else(|| format!("Missing lhs in {:?}", op))?
                    else {
                        return Err("Cannot do arithmetics on closures".into());
                    };

                    self.push(
                        ASObj::ASBooleen(match binop {
                            BinCompcode::Eq => arg1 == arg2,
                            BinCompcode::NotEq => arg1 != arg2,
                            BinCompcode::Lth => arg1 < arg2,
                            BinCompcode::Gth => arg1 > arg2,
                            BinCompcode::Geq => arg1 >= arg2,
                            BinCompcode::Leq => arg1 <= arg2,
                            BinCompcode::Dans => arg2.contains(&arg1).map_err(|e| e.to_string())?,
                            BinCompcode::PasDans => {
                                !arg2.contains(&arg1).map_err(|e| e.to_string())?
                            }
                        })
                        .into(),
                    );
                }
                Opcode::Jump => {
                    let dist = fnc.code[frame.ip];
                    frame.ip += dist as usize + 1;
                }
                Opcode::JumpIfFalse => {
                    let dist = fnc.code[frame.ip];
                    frame.ip += 1;

                    let val = self.pop().expect("A value");

                    if let Value::ASObj(v) = val {
                        if !v.to_bool() {
                            self.get_frame().unwrap().ip += dist as usize;
                        }
                    }
                }
            }
        }
    }
}
