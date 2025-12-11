use std::fmt::Debug;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use thiserror::Error;

use crate::{
    ast::{BinCompcode, BinOpcode},
    compiler::{bitmasks::BitArray, utils::format_table},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum Opcode {
    Constant,
    Closure,
    GetUpvalue,
    SetUpvalue,
    GetLocal,
    SetLocal,
    GetGlobal,
    SetGlobal,
    Call,

    Jump,
    JumpIfFalse,

    Return,
    Pop,

    BinOp,
    BinComp,
}

impl Opcode {
    pub const fn name(&self) -> &'static str {
        match self {
            Opcode::Constant => "CONST",
            Opcode::Closure => "CLOSURE",
            Opcode::GetUpvalue => "GET_UPVAL",
            Opcode::SetUpvalue => "SET_UPVAL",
            Opcode::GetLocal => "GET_LOCAL",
            Opcode::SetLocal => "SET_LOCAL",
            Opcode::GetGlobal => "GET_GLOBAL",
            Opcode::SetGlobal => "SET_GLOBAL",
            Opcode::Call => "CALL",
            Opcode::Return => "RETURN",
            Opcode::Pop => "POP",
            Opcode::BinOp => "BINOP",
            Opcode::BinComp => "BINCOMP",
            Opcode::Jump => "JUMP",
            Opcode::JumpIfFalse => "JUMP_IF_FALSE",
        }
    }

    pub const fn nargs(&self) -> u16 {
        match self {
            Opcode::Constant
            | Opcode::GetUpvalue
            | Opcode::SetUpvalue
            | Opcode::GetLocal
            | Opcode::SetLocal
            | Opcode::GetGlobal
            | Opcode::SetGlobal => 1,

            Opcode::Jump | Opcode::JumpIfFalse => 1,

            Opcode::Closure => 1,
            Opcode::Call => 1,
            Opcode::Return => todo!(),

            Opcode::BinOp | Opcode::BinComp => 1,

            Opcode::Pop => 0,
        }
    }
}

#[derive(Clone)]
pub struct Instructions {
    insts: Vec<u16>,
    opcodes: Vec<Opcode>,
}

pub fn instructions_to_string(insts: &[u16]) -> Vec<String> {
    let mut instructions = vec![];
    let mut iter = insts.iter();

    let mut op_i = 1;
    while let Some(byte) = iter.next() {
        let Ok(op) = Opcode::try_from(*byte) else {
            panic!("Invalid opcode {}", byte);
        };

        let mut inst_str = vec![];

        inst_str.push(format!("{}. {}", op_i, op.name()));

        match op {
            Opcode::Constant
            | Opcode::SetLocal
            | Opcode::GetLocal
            | Opcode::GetUpvalue
            | Opcode::SetUpvalue
            | Opcode::GetGlobal
            | Opcode::SetGlobal
            | Opcode::Closure
            | Opcode::Call => {
                let Some(idx) = iter.next() else {
                    panic!("Missing arg for {}", op.name());
                };

                inst_str.push(idx.to_string());
            }

            Opcode::JumpIfFalse | Opcode::Jump => {
                let Some(idx) = iter.next() else {
                    panic!("Missing arg for {}", op.name());
                };

                inst_str.push((*idx as i16 - JUMP_OFFSET).to_string());
                // inst_str.push(format!("(to {})", op_i + idx));
            }

            Opcode::BinOp => {
                let Some(op) = iter.next() else {
                    panic!("Missing arg for {}", op.name());
                };

                let binop = BinOpcode::try_from(*op).expect(&format!("Invalid binop: {}", op));

                inst_str.push(op.to_string());
                inst_str.push(format!("({:?})", binop));
            }
            Opcode::BinComp => {
                let Some(op) = iter.next() else {
                    panic!("Missing arg for {}", op.name());
                };

                let binop = BinCompcode::try_from(*op).expect(&format!("Invalid bin comp: {}", op));

                inst_str.push(op.to_string());
                inst_str.push(format!("({:?})", binop));
            }
            _ => {}
        }

        instructions.push(inst_str);

        op_i += 1;
    }

    let instructions = format_table(&instructions);

    instructions
}

impl Debug for Instructions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Instructions({})",
            format!(
                "[\n\t{}\n]",
                instructions_to_string(&self.insts).join("\n\t")
            )
        )
    }
}

impl Instructions {
    pub fn new() -> Self {
        Self {
            insts: vec![],
            opcodes: vec![],
        }
    }

    pub fn inner(&self) -> &Vec<u16> {
        &self.insts
    }

    pub fn raw_patch(&mut self, idx: usize, val: u16) {
        self.insts[idx] = val
    }

    fn emit_byte(&mut self, b: u16) {
        self.insts.push(b.into());
    }

    fn emit_opcode(&mut self, b: Opcode) {
        self.opcodes.push(b);
        self.emit_byte(b.into());
    }

    pub fn pop(&mut self) {
        let Some(last_op) = self.opcodes.last().copied() else {
            return;
        };

        self.opcodes.pop();

        // num args + the op
        for i in 0..=last_op.nargs() {
            self.insts.pop();
        }
    }

    pub fn pop_if_op_is(&mut self, op: Opcode) {
        if self.last_op_is(op) {
            self.pop();
        }
    }

    pub fn last_op_is(&self, op: Opcode) -> bool {
        self.opcodes.last().is_some_and(|lo| *lo == op)
    }

    pub fn emit_const(&mut self, idx: u16) {
        self.emit_opcode(Opcode::Constant);
        self.emit_byte(idx);
    }

    pub fn emit_closure(&mut self, const_idx: u16) {
        self.emit_opcode(Opcode::Closure);
        self.emit_byte(const_idx);
    }

    pub fn emit_get_upvalue(&mut self, idx: u16) {
        self.emit_opcode(Opcode::GetUpvalue);
        self.emit_byte(idx);
    }

    pub fn emit_set_upvalue(&mut self, idx: u16) {
        self.emit_opcode(Opcode::SetUpvalue);
        self.emit_byte(idx);
    }

    pub fn emit_get_local(&mut self, slot: u16) {
        self.emit_opcode(Opcode::GetLocal);
        self.emit_byte(slot);
    }

    pub fn emit_set_local(&mut self, slot: u16) {
        self.emit_opcode(Opcode::SetLocal);
        self.emit_byte(slot);
    }

    pub fn emit_get_global(&mut self, const_name_slot: u16) {
        self.emit_opcode(Opcode::GetGlobal);
        self.emit_byte(const_name_slot);
    }

    pub fn emit_set_global(&mut self, const_name_slot: u16) {
        self.emit_opcode(Opcode::SetGlobal);
        self.emit_byte(const_name_slot);
    }

    pub fn emit_call(&mut self, nargs: u16) {
        self.emit_opcode(Opcode::Call);
        self.emit_byte(nargs);
    }

    pub fn emit_return(&mut self) {
        self.emit_opcode(Opcode::Return);
    }

    pub fn emit_pop(&mut self) {
        self.emit_opcode(Opcode::Pop);
    }

    pub fn emit_binop(&mut self, op: BinOpcode) {
        self.emit_opcode(Opcode::BinOp);
        // The BinOpcode u16 value represent the operation done
        self.emit_byte(op as u16);
    }

    pub fn emit_bincomp(&mut self, op: BinCompcode) {
        self.emit_opcode(Opcode::BinComp);
        // The BinOpcode u16 value represent the operation done
        self.emit_byte(op as u16);
    }

    pub fn emit_jump(&mut self, target: i16) {
        self.emit_opcode(Opcode::Jump);
        self.emit_byte((target + JUMP_OFFSET) as u16);
    }

    pub fn emit_jump_if_false(&mut self, target: i16) {
        self.emit_opcode(Opcode::JumpIfFalse);
        self.emit_byte((target + JUMP_OFFSET) as u16);
    }
}

pub const JUMP_OFFSET: i16 = (1 << 8) - 1;

#[derive(Debug, Error)]
pub enum InstructionError {
    #[error("Invalid opcode: {0}")]
    InvalidOpcode(u16),
}
