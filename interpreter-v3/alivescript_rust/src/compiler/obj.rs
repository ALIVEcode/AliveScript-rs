use std::cell::RefCell;
use std::rc::Rc;

use crate::as_obj::ASObj;
use crate::compiler::bytecode::Instructions;
use crate::compiler::vm::VM;

pub type RcUpvalue = Rc<RefCell<Upvalue>>;
pub type RcClosure = Rc<Closure>;
pub type RcFunction = Rc<Function>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    ASObj(ASObj),
    Closure(RcClosure),
}

impl From<ASObj> for Value {
    fn from(value: ASObj) -> Self {
        Self::ASObj(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: Option<String>,
    pub code: Vec<u8>,         // bytecode
    pub constants: Vec<Value>, // constant pool
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

#[derive(Debug, PartialEq)]
pub struct Closure {
    pub function: Rc<Function>,
    pub upvalues: Vec<Rc<RefCell<Upvalue>>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum UpvalueLocation {
    Open(usize),                // index into VM.stack
    Closed(Rc<RefCell<Value>>), // heap cell
}

#[derive(Debug, PartialEq, Clone)]
pub struct Upvalue {
    pub location: UpvalueLocation,
}

impl Upvalue {
    pub fn get(&self, vm: &VM) -> Value {
        match &self.location {
            UpvalueLocation::Open(idx) => vm.stack[*idx].clone(),
            UpvalueLocation::Closed(cell) => cell.borrow().clone(),
        }
    }
    pub fn set(&mut self, vm: &mut VM, v: Value) {
        match &mut self.location {
            UpvalueLocation::Open(idx) => vm.stack[*idx] = v,
            UpvalueLocation::Closed(cell) => *cell.borrow_mut() = v,
        }
    }

    pub fn close(&mut self, vm: &VM) {
        if let UpvalueLocation::Open(idx) = self.location {
            let v = vm.stack[idx].clone();
            self.location = UpvalueLocation::Closed(Rc::new(RefCell::new(v)));
        }
    }
}

#[derive(Debug)]
pub struct CallFrame {
    pub closure: Rc<Closure>,
    pub ip: usize,
    pub base: usize, // where this frame's locals start in VM.stack
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpvalueSpec {
    Local(usize),   // refers to local slot index in the parent function
    Upvalue(usize), // refers to parent's upvalue number (chain)
}
