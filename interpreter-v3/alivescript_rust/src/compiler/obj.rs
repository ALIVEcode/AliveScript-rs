use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::rc::Rc;

use crate::as_obj::{self, ASErreurType, ASObj, ASType};
use crate::ast::CallRust;
use crate::compiler::bytecode::{instructions_to_string, Instructions};
use crate::compiler::vm::VM;

pub type RcUpvalue = Rc<RefCell<Upvalue>>;
pub type RcClosure = Rc<Closure>;
pub type RcFunction = Rc<Function>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    ASObj(ASObj),
    Closure(RcClosure),
    NativeFunction(NativeFunction),
}

impl Value {
    pub fn get_type(&self) -> ASType {
        match self {
            Value::ASObj(asobj) => asobj.get_type(),
            Value::Closure(closure) => ASType::Fonction,
            Value::NativeFunction(native_function) => ASType::Fonction,
        }
    }
}

impl From<ASObj> for Value {
    fn from(value: ASObj) -> Self {
        Self::ASObj(value)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let to_str = match self {
            Value::ASObj(asobj) => asobj.to_string(),
            Value::Closure(closure) => format!(
                "fonction {}()",
                closure
                    .function
                    .name
                    .as_ref()
                    .unwrap_or(&"anonyme".to_string())
            ),
            Value::NativeFunction(native_function) => {
                format!("fonction native {}()", native_function.name)
            }
        };

        write!(f, "{}", to_str)
    }
}

#[derive(Clone, PartialEq)]
pub struct Function {
    pub name: Option<String>,
    pub code: Vec<u16>,        // bytecode
    pub constants: Vec<Value>, // constant pool
    pub upvalue_count: usize,
    pub upvalue_specs: Vec<UpvalueSpec>, // from compiler: local? index?

    pub nb_params: usize,
}

impl Function {
    pub fn new(name: Option<String>, nb_params: usize) -> Self {
        Self {
            name,
            code: vec![],
            constants: vec![],
            upvalue_count: 0,
            upvalue_specs: vec![],
            nb_params: nb_params,
        }
    }

    pub fn new_anonymous(nb_params: usize) -> Self {
        Self {
            name: None,
            code: vec![],
            constants: vec![],
            upvalue_count: 0,
            upvalue_specs: vec![],

            nb_params,
        }
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function")
            .field("name", &self.name)
            .field("code", &instructions_to_string(&self.code))
            .field("constants", &self.constants)
            .field("upvalue_count", &self.upvalue_count)
            .field("upvalue_specs", &self.upvalue_specs)
            .finish()
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

impl UpvalueSpec {
    pub fn is_local(&self) -> bool {
        matches!(self, Self::Local(..))
    }

    pub fn index(&self) -> usize {
        match self {
            UpvalueSpec::Local(index) => *index,
            UpvalueSpec::Upvalue(index) => *index,
        }
    }
}

pub struct NativeFunction {
    pub func: Rc<dyn Fn(&mut VM) -> Result<Option<Value>, ASErreurType>>,
    pub name: Rc<String>,
    pub desc: Rc<Option<String>>,
}

impl Clone for NativeFunction {
    fn clone(&self) -> Self {
        Self {
            func: Rc::clone(&self.func),
            desc: Rc::clone(&self.desc),
            name: Rc::clone(&self.name),
        }
    }
}
impl PartialEq for NativeFunction {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}
impl std::fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("définition interne")
    }
}
