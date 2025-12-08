use std::cell::RefCell;
use std::rc::Rc;

use crate::as_obj::ASObj;
use crate::compiler::bytecode::Instructions;

pub type RcUpvalue = Rc<RefCell<Upvalue>>;
pub type RcClosure = Rc<Closure>;
pub type RcFunction = Rc<Function>;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Option<String>,
    pub code: Vec<u8>,    // bytecode
    pub constants: Vec<ASObj>, // constant pool
    pub upvalue_count: usize,
    pub upvalue_specs: Vec<UpvalueSpec>, // from compiler: local? index?
}

impl Function {
    pub fn new_anonymous() -> Self {
        Self {
            name: None,
            code: vec![],
            constants: vec![],
            upvalue_count: 0,
            upvalue_specs: vec![],
        }
    }
}

#[derive(Debug)]
pub struct Closure {
    pub function: Rc<Function>,
    pub upvalues: Vec<Rc<RefCell<Upvalue>>>,
}

#[derive(Debug)]
pub enum UpvalueLocation {
    Open(usize),                // index into VM.stack
    Closed(Rc<RefCell<ASObj>>), // heap cell
}

#[derive(Debug)]
pub struct Upvalue {
    pub location: UpvalueLocation,
}

#[derive(Debug)]
pub struct CallFrame {
    pub closure: Rc<Closure>,
    pub ip: usize,
    pub base: usize, // where this frame's locals start in VM.stack
}

#[derive(Debug, Clone, Copy)]
pub enum UpvalueSpec {
    Local(usize),   // refers to local slot index in the parent function
    Upvalue(usize), // refers to parent's upvalue number (chain)
}
