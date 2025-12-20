use std::{cell::RefCell, fmt::Debug, rc::Rc};

use num_enum::{IntoPrimitive, TryFromPrimitive};
use thiserror::Error;

use crate::compiler::{Compiler, bitmasks::BitArray, utils::format_table};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum Opcode {
    /// repush the top of the stack `push(top())`
    Dup,
    /// pop the top of the stack
    Pop,

    Constant,
    Closure,

    Read,
    /// stack: `msg`
    ReadWithMsg,
    /// stack: `func`
    ReadCall,
    /// stack: `msg, func`
    ReadCallWithMsg,

    GetUpvalue,
    SetUpvalue,
    GetLocal,
    /// stack: `value`
    SetLocal,
    GetGlobal,
    /// stack: `value`
    SetGlobal,
    Call,

    Jump,
    JumpIfFalse,

    Return,

    BinOp,
    Neg,
    BinComp,

    NewList,
    GetItem,
    SetItem,

    NewStruct,
    GetField,
    SetField,

    LoadModule,

    // ForPrep,
    // ForNext,
}

impl Opcode {
    pub const fn name(&self) -> &'static str {
        match self {
            Opcode::Dup => "DUP",
            Opcode::Pop => "POP",
            Opcode::Constant => "CONST",
            Opcode::Closure => "CLOSURE",
            Opcode::Read => "READ",
            Opcode::ReadWithMsg => "READ_MSG",
            Opcode::ReadCall => "READ_CALL",
            Opcode::ReadCallWithMsg => "READ_CALL_MSG",
            Opcode::GetUpvalue => "GET_UPVAL",
            Opcode::SetUpvalue => "SET_UPVAL",
            Opcode::GetLocal => "GET_LOCAL",
            Opcode::SetLocal => "SET_LOCAL",
            Opcode::GetGlobal => "GET_GLOBAL",
            Opcode::SetGlobal => "SET_GLOBAL",
            Opcode::Call => "CALL",
            Opcode::Return => "RETURN",
            Opcode::BinOp => "BINOP",
            Opcode::BinComp => "BINCOMP",
            Opcode::Neg => "NEG",
            Opcode::Jump => "JUMP",
            Opcode::JumpIfFalse => "JUMP_IF_FALSE",
            Opcode::NewList => "NEW_LIST",
            Opcode::GetItem => "GET_ITEM",
            Opcode::SetItem => "SET_ITEM",
            Opcode::NewStruct => "NEW_STRUCT",
            Opcode::GetField => "GET_ATTR",
            Opcode::SetField => "SET_ATTR",
            Opcode::LoadModule => "LOAD_MODULE",
            // Opcode::ForPrep => "FOR_PREP",
            // Opcode::ForNext => "FOR_NEXT",
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
            | Opcode::SetGlobal
            | Opcode::NewList
            | Opcode::NewStruct => 1,

            Opcode::Read => 0,
            // stack: [msg]
            Opcode::ReadWithMsg => 0,

            // stack: [func]
            Opcode::ReadCall => 1,
            // stack: [msg, func]
            Opcode::ReadCallWithMsg => 1,

            Opcode::Jump | Opcode::JumpIfFalse => 1,

            Opcode::Closure => 1,
            Opcode::Call => 1,

            Opcode::Return => 0,

            Opcode::BinOp | Opcode::BinComp => 1,

            Opcode::Dup | Opcode::Pop => 0,

            Opcode::GetItem | Opcode::Neg => 0,

            Opcode::SetItem => 0,

            Opcode::GetField => 1,
            Opcode::SetField => 1,

            Opcode::LoadModule => 1,
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
        let args = (0..op.nargs())
            .map(|_| {
                iter.next()
                    .expect(&format!("Missing arg for {}", op.name()))
            })
            .collect::<Vec<_>>();

        inst_str.extend(args.iter().map(|arg| arg.to_string()));

        match op {
            Opcode::JumpIfFalse | Opcode::Jump | Opcode::ReadCall | Opcode::ReadCallWithMsg => {
                let idx = args[0];

                inst_str.push((*idx as i16 - JUMP_OFFSET).to_string());
                // inst_str.push(format!("(to {})", op_i + idx));
            }

            Opcode::BinOp => {
                let op = args[0];
                let binop = BinOpcode::try_from(*op).expect(&format!("Invalid binop: {}", op));

                inst_str.push(format!("({:?})", binop));
            }
            Opcode::BinComp => {
                let op = args[0];
                let binop = BinCompcode::try_from(*op).expect(&format!("Invalid bin comp: {}", op));

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

pub fn instructions_to_string_debug(insts: &[u16], compiler: Rc<RefCell<Compiler>>) -> Vec<String> {
    let mut instructions = vec![];
    let mut iter = insts.iter();

    let mut op_i = 1;
    while let Some(byte) = iter.next() {
        let Ok(op) = Opcode::try_from(*byte) else {
            panic!("Invalid opcode {}", byte);
        };

        let mut inst_str = vec![];

        inst_str.push(format!("{}. {}", op_i, op.name()));
        let args = (0..op.nargs())
            .map(|_| {
                iter.next()
                    .expect(&format!("Missing arg for {}", op.name()))
            })
            .collect::<Vec<_>>();

        inst_str.extend(args.iter().map(|arg| arg.to_string()));

        match op {
            Opcode::Constant => {
                let idx = args[0];
                inst_str.push(format!(
                    "{}",
                    compiler.borrow().function.borrow().constants[*idx as usize]
                ));
            }
            Opcode::GetLocal | Opcode::SetLocal => {
                let idx = args[0];
                inst_str.push(format!(
                    "{:?}",
                    compiler.borrow().locals[*idx as usize].name
                ));
            }
            Opcode::GetUpvalue | Opcode::SetUpvalue => {
                let idx = args[0];
                inst_str.push(format!("{:?}", compiler.borrow().upvalues[*idx as usize]));
            }

            Opcode::JumpIfFalse | Opcode::Jump | Opcode::ReadCall | Opcode::ReadCallWithMsg => {
                let idx = args[0];

                inst_str.pop();
                inst_str.push(format!("{}", (*idx as i16 - JUMP_OFFSET)));
                // inst_str.push(format!("(to {})", op_i + idx));
            }

            Opcode::BinOp => {
                let op = args[0];
                let binop = BinOpcode::try_from(*op).expect(&format!("Invalid binop: {}", op));

                inst_str.push(format!("{:?}", binop));
            }
            Opcode::BinComp => {
                let op = args[0];
                let binop = BinCompcode::try_from(*op).expect(&format!("Invalid bin comp: {}", op));

                inst_str.push(format!("{:?}", binop));
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

    pub fn emit_neg(&mut self) {
        self.emit_opcode(Opcode::Neg);
    }

    pub fn emit_new_list(&mut self, nb_el: u16) {
        self.emit_opcode(Opcode::NewList);
        self.emit_byte(nb_el);
    }

    pub fn emit_new_struct(&mut self, nb_el: u16) {
        self.emit_opcode(Opcode::NewStruct);
        self.emit_byte(nb_el);
    }

    pub fn emit_get_item(&mut self) {
        self.emit_opcode(Opcode::GetItem);
    }

    pub fn emit_set_item(&mut self) {
        self.emit_opcode(Opcode::SetItem);
    }

    pub fn emit_get_attr(&mut self, const_idx: u16) {
        self.emit_opcode(Opcode::GetField);
        self.emit_byte(const_idx);
    }

    pub fn emit_set_attr(&mut self, const_idx: u16) {
        self.emit_opcode(Opcode::SetField);
        self.emit_byte(const_idx);
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

    pub fn emit_dup(&mut self) {
        self.emit_opcode(Opcode::Dup);
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

    pub fn emit_load_module(&mut self, module_name_const: u16) {
        self.emit_opcode(Opcode::LoadModule);
        self.emit_byte(module_name_const);
    }

    pub fn emit_read(&mut self, with_msg: bool) {
        if with_msg {
            self.emit_opcode(Opcode::ReadWithMsg);
        } else {
            self.emit_opcode(Opcode::Read);
        }
    }
    pub fn emit_read_call(&mut self, jmp: u16, with_msg: bool) {
        if with_msg {
            self.emit_opcode(Opcode::ReadCallWithMsg);
        } else {
            self.emit_opcode(Opcode::ReadCall);
        }
        self.emit_byte(jmp);
    }
}

pub const JUMP_OFFSET: i16 = (1 << 8) - 1;

#[derive(Debug, Error)]
pub enum InstructionError {
    #[error("Invalid opcode: {0}")]
    InvalidOpcode(u16),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnaryOpcode {
    Pas,
    Negate,
    Positive,
}

#[derive(Debug, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(u16)]
pub enum BinOpcode {
    Mul,
    Div,
    DivInt,
    Add,
    Sub,
    Exp,
    Mod,
    Extend,
    BitwiseOr,
    BitwiseAnd,
    BitwiseXor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Debug, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(u16)]
pub enum BinCompcode {
    Eq,
    NotEq,
    Lth,
    Gth,
    Geq,
    Leq,
    Dans,
    PasDans,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinLogiccode {
    Et,
    Ou,
    NonNul,
}
