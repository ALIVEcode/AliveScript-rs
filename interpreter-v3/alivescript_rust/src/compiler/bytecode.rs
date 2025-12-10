use std::fmt::Debug;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use thiserror::Error;

use crate::{
    ast::BinOpcode,
    compiler::{bitmasks::BitArray, utils::format_table},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Opcode {
    Constant,
    Closure,
    GetUpvalue,
    SetUpvalue,
    GetLocal,
    SetLocal,
    Call,

    Jump,
    JumpIfFalse,

    Return,
    Pop,

    BinOp,
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
            Opcode::Call => "CALL",
            Opcode::Return => "RETURN",
            Opcode::Pop => "POP",
            Opcode::BinOp => "BINOP",
            Opcode::Jump => "JUMP",
            Opcode::JumpIfFalse => "JUMP_IF_FALSE",
        }
    }

    pub const fn nargs(&self) -> u8 {
        match self {
            Opcode::Constant
            | Opcode::GetUpvalue
            | Opcode::SetUpvalue
            | Opcode::GetLocal
            | Opcode::SetLocal => 2,

            Opcode::Jump | Opcode::JumpIfFalse => 2,

            Opcode::Closure => todo!(),
            Opcode::Call => todo!(),
            Opcode::Return => todo!(),

            Opcode::BinOp => 1,

            Opcode::Pop => 0,
        }
    }
}

#[derive(Clone)]
pub struct Instructions {
    insts: Vec<u8>,
    opcodes: Vec<Opcode>,
}

impl Debug for Instructions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut instructions = vec![];
        let mut iter = self.insts.iter();

        while let Some(byte) = iter.next() {
            let Ok(op) = Opcode::try_from(*byte) else {
                panic!("Invalid opcode {}", byte);
            };

            let mut inst_str = vec![];

            inst_str.push(String::from(op.name()));

            match op {
                Opcode::Constant | Opcode::SetLocal | Opcode::GetLocal => {
                    let Some(idx) = iter.next() else {
                        panic!("Missing arg for {}", op.name());
                    };

                    inst_str.push(idx.to_string());
                }

                Opcode::BinOp => {
                    let Some(op) = iter.next() else {
                        panic!("Missing arg for {}", op.name());
                    };

                    let binop = BinOpcode::try_from(*op).expect(&format!("Invalid binop: {}", op));

                    inst_str.push(op.to_string());
                    inst_str.push(format!("({:?})", binop));
                }
                _ => {}
            }

            instructions.push(inst_str);
        }

        let instructions = format_table(&instructions);

        write!(f, "Instructions([\n\t{}\n])", instructions.join("\n\t"))
    }
}

impl Instructions {
    pub fn new() -> Self {
        Self {
            insts: vec![],
            opcodes: vec![],
        }
    }

    pub fn inner(&self) -> &Vec<u8> {
        &self.insts
    }

    fn emit_byte(&mut self, b: u8) {
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

    pub fn emit_const(&mut self, idx: u8) {
        self.emit_opcode(Opcode::Constant);
        self.emit_byte(idx);
    }

    pub fn emit_closure(&mut self, const_idx: u8) {
        self.emit_opcode(Opcode::Closure);
        self.emit_byte(const_idx);
    }

    pub fn emit_get_upvalue(&mut self, idx: u8) {
        self.emit_opcode(Opcode::GetUpvalue);
        self.emit_byte(idx);
    }

    pub fn emit_set_upvalue(&mut self, idx: u8) {
        self.emit_opcode(Opcode::SetUpvalue);
        self.emit_byte(idx);
    }

    pub fn emit_get_local(&mut self, slot: u8) {
        self.emit_opcode(Opcode::GetLocal);
        self.emit_byte(slot);
    }

    pub fn emit_set_local(&mut self, slot: u8) {
        self.emit_opcode(Opcode::SetLocal);
        self.emit_byte(slot);
    }

    pub fn emit_call(&mut self, nargs: u8) {
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
        // The BinOpcode u8 value represent the operation done
        self.emit_byte(op as u8);
    }
}

#[derive(Debug, Error)]
pub enum InstructionError {
    #[error("Invalid opcode: {0}")]
    InvalidOpcode(u8),
}
