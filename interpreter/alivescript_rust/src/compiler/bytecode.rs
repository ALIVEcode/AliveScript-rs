use std::{cell::RefCell, fmt::Debug, rc::Rc};

use num_enum::{IntoPrimitive, TryFromPrimitive};
use thiserror::Error;

use crate::compiler::{Compiler, bitmasks::BitArray, utils::format_table};

type OpcodeByteSize = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum Opcode {
    /// stack: `[any]`
    /// 1. repush the top of the stack `push(top())`
    Dup,

    /// stack: `[any]`
    /// 1. pop the top of the stack
    Pop,

    /// arg: const_idx
    /// 1. it gets the constant at `const_idx` (`const_val`)
    /// 2. it pushes the constant on the stack
    Constant,

    /// arg: const_idx
    /// 1. it gets the proto closure constant at `const_idx` (`closure_val`)
    /// 2. it closes the proto closure (turns it into a closure instance with resolved upvalue)
    /// 3. it pushes the closure instance on the stack
    Closure,

    Read,
    /// stack: `[msg]`
    ReadWithMsg,
    /// stack: `[func]`
    ReadCall,
    /// stack: `[msg, func]`
    ReadCallWithMsg,

    /// args: jump_to_catch_dist, nb_func_args
    /// stack: `[func, ...args]`
    /// 1. it calls the func with `nb_func_args`
    /// 2. if the call fails:
    ///     restore the stack to what it was
    ///     jump `jump_to_catch_dist`
    /// 3. else:
    ///     continue execution (do nothing)
    TryCall,

    GetUpvalue,
    SetUpvalue,
    GetLocal,
    /// stack: `[value]`
    SetLocal,
    GetGlobal,
    /// stack: `[value]`
    SetGlobal,
    Call,

    /// args: dist (i16)
    /// 1. jumps `dist`
    Jump,

    /// args: dist (i16)
    /// stack: `[val]`
    /// 1. it pops the top of the stack (`val`)
    /// 2. if `!val.to_bool()` -> jumps `dist`
    JumpIfFalse,

    /// args: dist (i16), cond (bool)
    /// stack: `[val]`
    /// 1. it peeks at the top of the stack (`val`)
    /// 2. if `val.to_bool() == cond` -> jumps `dist`
    /// 3. else -> pops the top of the stack
    JumpTest,

    Return,

    BinOp,
    Neg,
    BinComp,
    Not,

    /// arg: nb_elements
    /// stack: `[el_n, ..., el_2, el_1]`
    /// 1. it pops `nb_elements` from the stack
    /// 2. it creates a new list
    /// 3. it pushes the list on the stack
    NewList,

    /// stack: `[obj, item]`
    /// 1. pops the item
    /// 2. pops the obj
    /// 3. pushes `obj[item]`
    GetItem,

    /// stack: `[value, obj, item]`
    /// 1. pops the item
    /// 2. pops the obj
    /// 3. pops the value
    /// 4. does `obj[item] = value`
    SetItem,

    /// arg: nb_fields
    /// stack: `[struct, field_n_value, field_n_name, ..., field_2_value, field_2_name, field_1_value, field_1_name]`
    /// 1. it pops the name and the value `nb_fields` times
    /// 2. it pops the struct
    /// 3. it populates a new instance of the struct with the fields and resolve the
    /// default values for the unspecified fields (or throws an error)
    /// 3. it pushes the new instance on the stack
    NewStruct,

    /// arg: const_name_idx
    /// stack: `[obj]`
    /// 1. it gets the text constant at `const_name_idx` (`field_name`)
    /// 2. pops the obj
    /// 3. pushes `obj.field_name`
    GetField,

    /// arg: const_name_idx
    /// stack: `[value, obj]`
    /// 1. it gets the text constant at `const_name_idx` (`field_name`)
    /// 2. pops the obj
    /// 3. pops the value
    /// 3.5 Checks if `obj.field_name` can be assigned
    /// 4. does `obj.field_name = value`
    SetField,

    /// arg: const_name_idx
    /// stack: `[struct, closure]`
    /// 1. it gets the text constant at `const_name_idx` (`field_name`)
    /// 2. it `pops` the closure
    /// 3. it `peeks` at the top of the stack to get the struct
    /// 4. it assigns the closure to the field with the name `field_name`
    SetDefaultField,

    /// arg: const_name_idx
    /// stack: `[struct, closure]`
    /// 1. it gets the text constant at `const_name_idx` (`method_name`)
    /// 2. it `pops` the closure
    /// 3. it `peeks` at the top of the stack to get the struct
    /// 4. it assigns the closure to the method with the name `method_name`
    SetMethod,

    /// arg: const_name_idx
    /// stack: `[]`
    /// 1. it gets the text constant at `const_name_idx` (`module_name`)
    /// 2. it tries to load the module with the name `module_name`
    /// 3. it pushes the loaded module on the stack
    LoadModule,

    /// arg: iter_idx
    /// iter_idx + 1 => iter_state
    /// updates iter_state and gets the next value for the next iteration
    ///
    /// if the next iteration is a valid value, then it :
    /// -> push(next)
    /// -> jump 1 (skips the jump {loop_len})
    ///
    /// if the next iteration is **not** a valid value, then it continues
    /// to the next instruction, which is a
    /// -> jump {loop_len}
    ForNext,
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
            Opcode::TryCall => "TRY_CALL",
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
            Opcode::Not => "NOT",
            Opcode::Neg => "NEG",
            Opcode::Jump => "JUMP",
            Opcode::JumpIfFalse => "JUMP_IF_FALSE",
            Opcode::JumpTest => "JUMP_TEST",
            Opcode::NewList => "NEW_LIST",
            Opcode::GetItem => "GET_ITEM",
            Opcode::SetItem => "SET_ITEM",
            Opcode::NewStruct => "NEW_STRUCT",
            Opcode::GetField => "GET_ATTR",
            Opcode::SetField => "SET_ATTR",
            Opcode::SetDefaultField => "SET_DEFAULT_FIELD",
            Opcode::SetMethod => "SET_METHOD",
            Opcode::LoadModule => "LOAD_MODULE",
            // Opcode::ForPrep => "FOR_PREP",
            Opcode::ForNext => "FOR_NEXT",
        }
    }

    pub const fn nargs(&self) -> OpcodeByteSize {
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

            Opcode::TryCall => 2,

            Opcode::Jump | Opcode::JumpIfFalse => 1,
            Opcode::JumpTest => 2,

            Opcode::ForNext => 1,

            Opcode::Closure => 1,
            Opcode::Call => 1,

            Opcode::Return => 0,

            Opcode::BinOp | Opcode::BinComp => 1,

            Opcode::Dup | Opcode::Pop => 0,

            Opcode::GetItem | Opcode::Neg | Opcode::Not => 0,

            Opcode::SetItem => 0,

            Opcode::GetField => 1,
            Opcode::SetField => 1,
            Opcode::SetMethod => 1,
            Opcode::SetDefaultField => 1,

            Opcode::LoadModule => 1,
        }
    }
}

#[derive(Clone)]
pub struct Instructions {
    insts: Vec<OpcodeByteSize>,
    opcodes: Vec<Opcode>,
    cursor: Option<usize>,
}

pub fn instructions_to_string(insts: &[OpcodeByteSize]) -> Vec<String> {
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

            Opcode::JumpTest => {
                let idx = args[0];
                let cond = args[1];

                inst_str.pop();
                inst_str.pop();
                inst_str.push(format!("{}", (*idx as i16 - JUMP_OFFSET)));
                inst_str.push(format!(
                    "{}",
                    if *cond == 1 { "if true" } else { "if false" }
                ));
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

pub fn instructions_to_string_debug(
    insts: &[OpcodeByteSize],
    compiler: Rc<RefCell<Compiler>>,
) -> Vec<String> {
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

            Opcode::TryCall => {
                let nargs = args[0];
                let idx = args[1];

                inst_str.pop();
                inst_str.pop();
                inst_str.push(format!("{}", nargs));
                inst_str.push(format!("{}", (*idx as i16 - JUMP_OFFSET)));
            }

            Opcode::JumpTest => {
                let idx = args[0];
                let cond = args[1];

                inst_str.pop();
                inst_str.pop();
                inst_str.push(format!("{}", (*idx as i16 - JUMP_OFFSET)));
                inst_str.push(format!(
                    "{}",
                    if *cond == 1 { "if true" } else { "if false" }
                ));
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
            cursor: None,
        }
    }

    pub fn len_inner(&self) -> usize {
        self.insts.len()
    }

    pub fn inner(&self) -> &Vec<OpcodeByteSize> {
        &self.insts
    }

    pub fn raw_patch(&mut self, idx: usize, val: OpcodeByteSize) {
        self.insts[idx] = val
    }

    pub fn set_cursor(&mut self, pos: usize) {
        self.cursor = Some(pos);
    }
    pub fn remove_cursor(&mut self) {
        self.cursor = None
    }

    fn emit_byte(&mut self, b: OpcodeByteSize) {
        if let Some(cursor) = &mut self.cursor {
            self.insts.insert(*cursor, b.into());
            *cursor += 1;
        } else {
            self.insts.push(b.into());
        }
    }

    fn emit_opcode(&mut self, b: Opcode) {
        if self.cursor.is_none() {
            self.opcodes.push(b);
        }
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

    pub fn emit_const(&mut self, idx: OpcodeByteSize) {
        self.emit_opcode(Opcode::Constant);
        self.emit_byte(idx);
    }

    pub fn emit_neg(&mut self) {
        self.emit_opcode(Opcode::Neg);
    }

    pub fn emit_not(&mut self) {
        self.emit_opcode(Opcode::Not);
    }

    pub fn emit_new_list(&mut self, nb_el: OpcodeByteSize) {
        self.emit_opcode(Opcode::NewList);
        self.emit_byte(nb_el);
    }

    pub fn emit_new_struct(&mut self, nb_el: OpcodeByteSize) {
        self.emit_opcode(Opcode::NewStruct);
        self.emit_byte(nb_el);
    }

    pub fn emit_get_item(&mut self) {
        self.emit_opcode(Opcode::GetItem);
    }

    pub fn emit_set_item(&mut self) {
        self.emit_opcode(Opcode::SetItem);
    }

    pub fn emit_get_attr(&mut self, const_idx: OpcodeByteSize) {
        self.emit_opcode(Opcode::GetField);
        self.emit_byte(const_idx);
    }

    pub fn emit_set_field(&mut self, const_idx: OpcodeByteSize) {
        self.emit_opcode(Opcode::SetField);
        self.emit_byte(const_idx);
    }

    pub fn emit_set_method(&mut self, name_const_idx: OpcodeByteSize) {
        self.emit_opcode(Opcode::SetMethod);
        self.emit_byte(name_const_idx);
    }

    pub fn emit_set_default_field(&mut self, name_const_idx: OpcodeByteSize) {
        self.emit_opcode(Opcode::SetDefaultField);
        self.emit_byte(name_const_idx);
    }

    pub fn emit_closure(&mut self, const_idx: OpcodeByteSize) {
        self.emit_opcode(Opcode::Closure);
        self.emit_byte(const_idx);
    }

    pub fn emit_get_upvalue(&mut self, idx: OpcodeByteSize) {
        self.emit_opcode(Opcode::GetUpvalue);
        self.emit_byte(idx);
    }

    pub fn emit_set_upvalue(&mut self, idx: OpcodeByteSize) {
        self.emit_opcode(Opcode::SetUpvalue);
        self.emit_byte(idx);
    }

    pub fn emit_get_local(&mut self, slot: OpcodeByteSize) {
        self.emit_opcode(Opcode::GetLocal);
        self.emit_byte(slot);
    }

    pub fn emit_set_local(&mut self, slot: OpcodeByteSize) {
        self.emit_opcode(Opcode::SetLocal);
        self.emit_byte(slot);
    }

    pub fn emit_get_global(&mut self, const_name_slot: OpcodeByteSize) {
        self.emit_opcode(Opcode::GetGlobal);
        self.emit_byte(const_name_slot);
    }

    pub fn emit_set_global(&mut self, const_name_slot: OpcodeByteSize) {
        self.emit_opcode(Opcode::SetGlobal);
        self.emit_byte(const_name_slot);
    }

    pub fn emit_call(&mut self, nargs: OpcodeByteSize) {
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
        // The BinOpcode OpcodeByteSize value represent the operation done
        self.emit_byte(op as OpcodeByteSize);
    }

    pub fn emit_bincomp(&mut self, op: BinCompcode) {
        self.emit_opcode(Opcode::BinComp);
        // The BinOpcode OpcodeByteSize value represent the operation done
        self.emit_byte(op as OpcodeByteSize);
    }

    pub fn emit_jump(&mut self, target: i16) {
        self.emit_opcode(Opcode::Jump);
        self.emit_byte((target + JUMP_OFFSET) as OpcodeByteSize);
    }

    pub fn emit_jump_test(&mut self, target: i16, cond: bool) {
        self.emit_opcode(Opcode::JumpTest);
        self.emit_byte((target + JUMP_OFFSET) as OpcodeByteSize);
        self.emit_byte(cond as OpcodeByteSize);
    }

    pub fn emit_jump_if_false(&mut self, target: i16) {
        self.emit_opcode(Opcode::JumpIfFalse);
        self.emit_byte((target + JUMP_OFFSET) as OpcodeByteSize);
    }

    pub fn emit_load_module(&mut self, module_name_const: OpcodeByteSize) {
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
    pub fn emit_read_call(&mut self, jmp: OpcodeByteSize, with_msg: bool) {
        if with_msg {
            self.emit_opcode(Opcode::ReadCallWithMsg);
        } else {
            self.emit_opcode(Opcode::ReadCall);
        }
        self.emit_byte(jmp);
    }
    pub fn emit_try_call(&mut self, jmp: OpcodeByteSize, nargs: OpcodeByteSize) {
        self.emit_opcode(Opcode::TryCall);
        self.emit_byte(nargs);
        self.emit_byte(jmp);
    }

    pub fn emit_for_next(&mut self, first_var_slot: OpcodeByteSize) {
        self.emit_opcode(Opcode::ForNext);
        self.emit_byte(first_var_slot);
    }
}

pub const JUMP_OFFSET: i16 = (1 << 8) - 1;

#[derive(Debug, Error)]
pub enum InstructionError {
    #[error("Invalid opcode: {0}")]
    InvalidOpcode(OpcodeByteSize),
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
