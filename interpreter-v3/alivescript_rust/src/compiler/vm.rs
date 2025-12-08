use std::{cell::RefCell, rc::Rc};

use crate::{
    as_obj::ASObj,
    compiler::{
        bytecode::Opcode,
        obj::{CallFrame, RcClosure, RcUpvalue, Upvalue},
    },
};

pub struct VM {
    pub stack: Vec<ASObj>,
    frames: Vec<CallFrame>,
    open_upvalues: Vec<RcUpvalue>, // track upvalues that point to stack slots
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            frames: Vec::new(),
            open_upvalues: Vec::new(),
        }
    }

    fn push(&mut self, v: ASObj) {
        self.stack.push(v);
    }

    fn pop(&mut self) -> Option<ASObj> {
        self.stack.pop()
    }

    fn peek(&self, distance: usize) -> &ASObj {
        let idx = self.stack.len() - 1 - distance;
        &self.stack[idx]
    }

    fn get_frame(&mut self) -> Result<&mut CallFrame, String> {
        self.frames.last_mut().ok_or("no frame".into())
    }

    pub fn run(&mut self, closure: RcClosure) -> Result<ASObj, String> {
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
                Opcode::Closure => todo!(),
                Opcode::GetUpvalue => todo!(),
                Opcode::SetUpvalue => todo!(),
                Opcode::GetLocal => {
                    let slot = fnc.code[frame.ip] as usize;
                    frame.ip += 1;
                    let idx = frame.base + slot;
                    let v = self.stack.get(idx).cloned().unwrap_or(ASObj::ASNoValue);
                    self.push(v);
                }
                Opcode::SetLocal => {
                    let slot = fnc.code[frame.ip] as usize;
                    frame.ip += 1;
                    let idx = frame.base + slot;

                    let val = self.pop().ok_or("Missing value in SET_LOCAL")?;
                    if idx >= self.stack.len() {
                        // expand stack to fit local (for simplicity)
                        self.stack.resize(idx + 1, ASObj::ASNoValue);
                    }
                    self.stack[idx] = val;
                }
                Opcode::Call => todo!(),
                Opcode::Return => return Ok(self.pop().unwrap_or(ASObj::ASNul)),
                Opcode::Pop => {
                    self.pop();
                }
                Opcode::Mul
                | Opcode::Div
                | Opcode::DivInt
                | Opcode::Add
                | Opcode::Sub
                | Opcode::Exp
                | Opcode::Mod
                | Opcode::Extend
                | Opcode::BitwiseOr
                | Opcode::BitwiseAnd
                | Opcode::BitwiseXor
                | Opcode::ShiftLeft
                | Opcode::ShiftRight => {
                    let arg2 = self
                        .pop()
                        .ok_or_else(|| format!("Missing rhs in {}", op.name()))?;
                    let arg1 = self
                        .pop()
                        .ok_or_else(|| format!("Missing lhs in {}", op.name()))?;

                    self.push(match op {
                        Opcode::Mul => arg1 * arg2,
                        Opcode::Div => arg1 / arg2,
                        Opcode::DivInt => arg1.div_int(arg2),
                        Opcode::Add => arg1 + arg2,
                        Opcode::Sub => arg1 - arg2,
                        Opcode::Exp => arg1.pow(arg2),
                        Opcode::Mod => arg1 % arg2,
                        Opcode::Extend => arg1.extend(arg2).map_err(|err| err.to_string())?,
                        Opcode::BitwiseOr => (arg1 | arg2).map_err(|err| err.to_string())?,
                        Opcode::BitwiseAnd => (arg1 & arg2).map_err(|err| err.to_string())?,
                        Opcode::BitwiseXor => (arg1 ^ arg2).map_err(|err| err.to_string())?,
                        Opcode::ShiftLeft => (arg1 << arg2).map_err(|err| err.to_string())?,
                        Opcode::ShiftRight => (arg1 >> arg2).map_err(|err| err.to_string())?,
                        _ => unreachable!(),
                    });
                }
            }
        }
    }
}
