use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub};
use std::sync::{Arc, Mutex, RwLock};

use crate::as_obj::{self, ASErreurType, ASType};
use crate::ast::CallRust;
use crate::compiler::bytecode::{Instructions, instructions_to_string};
use crate::compiler::obj::Value;
use crate::compiler::vm::VM;

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

#[derive(Debug)]
pub struct ASStructure {
    name: String,
}
