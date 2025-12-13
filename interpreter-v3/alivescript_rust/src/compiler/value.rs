use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub};
use std::sync::{Arc, Mutex, RwLock};

use crate::as_obj::{self, ASErreurType, ASType};
use crate::ast::CallRust;
use crate::compiler::bytecode::{Instructions, instructions_to_string};
use crate::compiler::obj::{ArcClosure, Upvalue, UpvalueSpec, Value};
use crate::compiler::vm::VM;

#[derive(Debug)]
pub struct ASStructure {
    name: String,

    fields: HashMap<String, ASFieldInfo>,
    methods: HashMap<String, NativeFunction>,
}

impl ASStructure {
    pub fn new(
        name: String,
        fields: HashMap<String, ASFieldInfo>,
        methods: HashMap<String, NativeFunction>,
    ) -> Self {
        Self {
            name,
            fields,
            methods,
        }
    }
}

#[derive(Debug)]
pub struct ASFieldInfo {
    field_type: ASType,
    is_const: bool,
    is_private: bool,
    value: Value,
}

#[derive(Debug)]
pub struct ClosureMethod {
    pub function: Arc<Function>,
    pub upvalues: Vec<Upvalue>,
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

#[derive(Debug)]
pub struct Closure {
    pub function: Arc<Function>,
    pub upvalues: Vec<Arc<RwLock<Upvalue>>>,
}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        self.function == other.function
            && self
                .upvalues
                .iter()
                .zip(other.upvalues.iter())
                .all(|(u1, u2)| Arc::ptr_eq(&u1, &u2))
    }
}
pub struct NativeFunction {
    pub func: Arc<dyn Fn(&mut VM, Vec<Value>) -> Result<Option<Value>, ASErreurType>>,
    pub name: Arc<String>,
    pub desc: Arc<Option<String>>,
}

impl Clone for NativeFunction {
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
            desc: Arc::clone(&self.desc),
            name: Arc::clone(&self.name),
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
        write!(f, "fonction native {}()", self.name)
    }
}

