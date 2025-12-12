use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub};
use std::rc::Rc;

use crate::as_obj::{self, ASErreurType, ASObj, ASType};
use crate::ast::CallRust;
use crate::compiler::bytecode::{instructions_to_string, Instructions};
use crate::compiler::vm::VM;

pub type RcUpvalue = Rc<RefCell<Upvalue>>;
pub type RcClosure = Rc<Closure>;
pub type RcFunction = Rc<Function>;

#[derive(Debug, Clone)]
pub enum Value {
    ASObj(ASObj),
    Closure(RcClosure),
    NativeFunction(NativeFunction),
    List(Rc<RefCell<Vec<Value>>>),
}

impl Value {
    pub fn get_type(&self) -> ASType {
        match self {
            Value::ASObj(asobj) => asobj.get_type(),
            Value::Closure(closure) => ASType::Fonction,
            Value::NativeFunction(native_function) => ASType::Fonction,
            Value::List(list) => ASType::Liste,
        }
    }

    pub fn div_int(&self, rhs: Self) -> Value {
        use ASObj::*;

        let Value::ASObj(s) = self else {
            unreachable!()
        };
        let Value::ASObj(r) = rhs else { unreachable!() };
        match (s, r) {
            (ASEntier(x), ASEntier(y)) => ASEntier(x / y),
            (ASDecimal(x), ASEntier(y)) => ASEntier(*x as i64 / y),
            (ASEntier(x), ASDecimal(y)) => ASEntier(x / y as i64),
            (ASDecimal(x), ASDecimal(y)) => ASEntier(*x as i64 / y as i64),
            _ => unimplemented!(),
        }
        .into()
    }

    pub fn pow(&self, rhs: Self) -> Value {
        use ASObj::*;

        let Value::ASObj(s) = self else {
            unreachable!()
        };
        let Value::ASObj(r) = rhs else { unreachable!() };
        match (s, r) {
            (ASEntier(x), ASEntier(y)) => ASEntier(x.pow(y as u32)),
            (ASDecimal(x), ASEntier(y)) => ASDecimal(x.powi(y as i32)),
            (ASEntier(x), ASDecimal(y)) => ASDecimal((*x as f64).powf(y)),
            (ASDecimal(x), ASDecimal(y)) => ASDecimal(x.powf(y)),
            _ => unimplemented!(),
        }
        .into()
    }

    pub fn extend(&self, rhs: Self) -> Result<Value, ASErreurType> {
        use ASObj::*;

        match (self, rhs) {
            (Value::ASObj(ASTexte(s)), rhs @ Value::ASObj(ASTexte(..))) => Ok(self.clone() + rhs),
            (Value::List(l), Value::List(l2)) => {
                let mut l3 = l.borrow().clone();
                l3.extend(l2.borrow().to_owned());
                Ok(Value::List(Rc::new(RefCell::new(l3))))
            }

            // (ASDict(d), ASDict(d2)) => {
            //     let mut d3 = d.borrow().clone();
            //     for e in d2.borrow().items() {
            //         d3.insert(e.key().to_owned(), e.val().to_owned());
            //     }
            //     Ok(ASObj::dict(d3))
            // }
            //
            // (ASTuple(_), _) => todo!("Tuple pas encore (et peut-être jamais) dans le langage"),
            // (ASClasse(classe), _) => todo!("Check présense du field?"),
            // (ASModule { name, alias, env }, _) => todo!(),
            (_, rhs) => Err(ASErreurType::new_erreur_operation(
                "++".into(),
                self.get_type(),
                rhs.get_type(),
            )),
        }
    }

    pub fn contains(&self, rhs: &Self) -> Result<bool, ASErreurType> {
        use ASObj::*;

        match (self, rhs) {
            (Value::ASObj(ASTexte(s)), Value::ASObj(ASTexte(sub_s))) => Ok(s.contains(sub_s)),
            (Value::List(l), rhs) => Ok(l.borrow().contains(rhs)),
            // (ASDict(d), rhs) => Ok(d.borrow().contains(rhs)),

            // (ASTuple(_), _) => todo!("Tuple pas encore (et peut-être jamais) dans le langage"),
            // (ASClasse(classe), _) => todo!("Check présense du field?"),
            // (ASModule { name, alias, env }, _) => todo!(),
            _ => Err(ASErreurType::new_erreur_operation(
                "dans".into(),
                self.get_type(),
                rhs.get_type(),
            )),
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
            Value::List(vals) => format!(
                "[{}]",
                vals.borrow()
                    .iter()
                    .map(|val| val.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
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

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use ASObj::*;

        match (self, other) {
            (Value::ASObj(ASEntier(i)), Value::ASObj(ASDecimal(d)))
            | (Value::ASObj(ASDecimal(d)), Value::ASObj(ASEntier(i))) => *d == *i as f64,
            (Value::ASObj(ASEntier(i1)), Value::ASObj(ASEntier(i2))) => i1 == i2,
            (Value::ASObj(ASTexte(t1)), Value::ASObj(ASTexte(t2))) => t1 == t2,
            (Value::ASObj(ASBooleen(b1)), Value::ASObj(ASBooleen(b2))) => b1 == b2,
            (Value::List(l1), Value::List(l2)) => {
                l1.borrow().as_ref() as &Vec<Value> == l2.borrow().as_ref() as &Vec<Value>
            }
            // (ASDict(d1), ASDict(d2)) => d1 == d2,
            // (ASFonc(f1), ASFonc(f2)) => f1 == f2,
            // (ASClasse(classe1), ASClasse(classe2)) => classe1 == classe2,
            // (ASClasseInst(inst1), ASClasseInst(inst2)) => inst1 == inst2,
            (Value::ASObj(ASNul), Value::ASObj(ASNul)) => true,
            (Value::ASObj(ASNoValue), Value::ASObj(ASNoValue)) => true,
            _ => false,
        }
    }
}

impl Add for Value {
    type Output = Value;

    fn add(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        match (self, rhs) {
            (Value::List(l), any) => Value::List({
                let mut l = l.borrow().clone();
                l.push(any);
                Rc::new(RefCell::new(l))
            }),
            (Value::ASObj(ASTexte(s)), any) => ASTexte(format!("{}{}", s, any.to_string())).into(),
            (any, Value::ASObj(ASTexte(s))) => ASTexte(format!("{}{}", any.to_string(), s)).into(),
            (Value::ASObj(ASEntier(x)), Value::ASObj(ASEntier(y))) => ASEntier(x + y).into(),
            (Value::ASObj(ASDecimal(x)), Value::ASObj(ASEntier(y))) => {
                ASDecimal(x + y as f64).into()
            }
            (Value::ASObj(ASEntier(x)), Value::ASObj(ASDecimal(y))) => {
                ASDecimal(x as f64 + y).into()
            }
            (Value::ASObj(ASDecimal(x)), Value::ASObj(ASDecimal(y))) => ASDecimal(x + y).into(),
            (l, r) => unimplemented!("Add for {} and {}", l, r),
        }
    }
}

impl Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        match (self, rhs) {
            (Value::ASObj(ASTexte(s)), Value::ASObj(ASTexte(s2))) => {
                ASTexte(s.replace(s2.as_str(), ""))
            }
            (Value::ASObj(ASEntier(x)), Value::ASObj(ASEntier(y))) => ASEntier(x - y),
            (Value::ASObj(ASDecimal(x)), Value::ASObj(ASEntier(y))) => ASDecimal(x - y as f64),
            (Value::ASObj(ASEntier(x)), Value::ASObj(ASDecimal(y))) => ASDecimal(x as f64 - y),
            (Value::ASObj(ASDecimal(x)), Value::ASObj(ASDecimal(y))) => ASDecimal(x - y),
            _ => unimplemented!(),
        }
        .into()
    }
}

impl Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        match (self, rhs) {
            (Value::ASObj(ASTexte(s)), Value::ASObj(ASEntier(n))) => {
                ASTexte(s.repeat(if n >= 0 { n as usize } else { 0 })).into()
            }
            (Value::ASObj(ASEntier(x)), Value::ASObj(ASEntier(y))) => ASEntier(x * y).into(),
            (Value::ASObj(ASDecimal(x)), Value::ASObj(ASEntier(y))) => {
                ASDecimal(x * y as f64).into()
            }
            (Value::ASObj(ASEntier(x)), Value::ASObj(ASDecimal(y))) => {
                ASDecimal(x as f64 * y).into()
            }
            (Value::ASObj(ASDecimal(x)), Value::ASObj(ASDecimal(y))) => ASDecimal(x * y).into(),
            (Value::List(l), Value::ASObj(ASEntier(n))) => Value::List(if n <= 0 {
                Rc::new(RefCell::new(vec![]))
            } else {
                let n = n as usize;
                let l = l.borrow();
                let len = l.len();
                let mut new_vec = Vec::with_capacity(n * len);
                for i in 0..n * len {
                    new_vec.push(l[i % len].clone());
                }
                Rc::new(RefCell::new(new_vec))
            }),
            _ => unimplemented!(),
        }
    }
}

impl Div for Value {
    type Output = Value;

    fn div(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        match (self, rhs) {
            (Value::ASObj(ASEntier(x)), Value::ASObj(ASEntier(y))) => {
                ASDecimal(x as f64 / y as f64).into()
            }
            (Value::ASObj(ASDecimal(x)), Value::ASObj(ASEntier(y))) => {
                ASDecimal(x / y as f64).into()
            }
            (Value::ASObj(ASEntier(x)), Value::ASObj(ASDecimal(y))) => {
                ASDecimal(x as f64 / y).into()
            }
            (Value::ASObj(ASDecimal(x)), Value::ASObj(ASDecimal(y))) => ASDecimal(x / y).into(),
            _ => unimplemented!(),
        }
    }
}

impl Rem for Value {
    type Output = Value;

    fn rem(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        match (self, rhs) {
            (Value::ASObj(ASEntier(x)), Value::ASObj(ASEntier(y))) => ASEntier((x % y + y) % y),
            (Value::ASObj(ASDecimal(x)), Value::ASObj(ASEntier(y))) => {
                ASDecimal((x % y as f64 + y as f64) % y as f64)
            }
            (Value::ASObj(ASEntier(x)), Value::ASObj(ASDecimal(y))) => {
                ASDecimal((x as f64 % y + y) % y)
            }
            (Value::ASObj(ASDecimal(x)), Value::ASObj(ASDecimal(y))) => ASDecimal((x % y + y) % y),
            _ => unimplemented!(),
        }
        .into()
    }
}

impl BitXor for Value {
    type Output = Result<Value, ASErreurType>;

    fn bitxor(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        let type_1 = self.get_type().clone();
        let type_2 = rhs.get_type().clone();

        let Value::ASObj(s) = self else {
            unreachable!()
        };
        let Value::ASObj(r) = rhs else { unreachable!() };
        match (s, r) {
            (ASEntier(x), ASEntier(y)) => Ok(ASEntier(x ^ y).into()),
            (ASBooleen(x), ASBooleen(y)) => Ok(ASBooleen(x ^ y).into()),
            _ => Err(ASErreurType::new_erreur_operation(
                "xor".into(),
                type_1,
                type_2,
            )),
        }
    }
}

impl BitAnd for Value {
    type Output = Result<Value, ASErreurType>;

    fn bitand(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        let type_1 = self.get_type().clone();
        let type_2 = rhs.get_type().clone();

        let Value::ASObj(s) = self else {
            unreachable!()
        };
        let Value::ASObj(r) = rhs else { unreachable!() };
        match (s, r) {
            (ASEntier(x), ASEntier(y)) => Ok(ASEntier(x & y).into()),
            (ASBooleen(x), ASBooleen(y)) => Ok(ASBooleen(x & y).into()),
            _ => Err(ASErreurType::new_erreur_operation(
                "&".into(),
                type_1,
                type_2,
            )),
        }
    }
}

impl BitOr for Value {
    type Output = Result<Value, ASErreurType>;

    fn bitor(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        let type_1 = self.get_type().clone();
        let type_2 = rhs.get_type().clone();
        let Value::ASObj(s) = self else {
            unreachable!()
        };
        let Value::ASObj(r) = rhs else { unreachable!() };
        match (s, r) {
            (ASEntier(x), ASEntier(y)) => Ok(ASEntier(x | y).into()),
            (ASBooleen(x), ASBooleen(y)) => Ok(ASBooleen(x | y).into()),
            _ => Err(ASErreurType::new_erreur_operation(
                "|".into(),
                type_1,
                type_2,
            )),
        }
    }
}

impl Shl for Value {
    type Output = Result<Value, ASErreurType>;

    fn shl(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        let type_1 = self.get_type().clone();
        let type_2 = rhs.get_type().clone();
        let Value::ASObj(s) = self else {
            unreachable!()
        };
        let Value::ASObj(r) = rhs else { unreachable!() };
        match (s, r) {
            (ASEntier(x), ASEntier(y)) => Ok(ASEntier(x << y).into()),
            _ => Err(ASErreurType::new_erreur_operation(
                "<<".into(),
                type_1,
                type_2,
            )),
        }
    }
}

impl Shr for Value {
    type Output = Result<Value, ASErreurType>;

    fn shr(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        let type_1 = self.get_type().clone();
        let type_2 = rhs.get_type().clone();
        let Value::ASObj(s) = self else {
            unreachable!()
        };
        let Value::ASObj(r) = rhs else { unreachable!() };
        match (s, r) {
            (ASEntier(x), ASEntier(y)) => Ok(ASEntier(x >> y).into()),
            _ => Err(ASErreurType::new_erreur_operation(
                ">>".into(),
                type_1,
                type_2,
            )),
        }
    }
}
